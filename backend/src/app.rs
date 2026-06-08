use crate::{
    db::Db,
    error::{ApiError, ApiResult},
    models::*,
};
use argon2::{Argon2, PasswordHasher};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::{get, post, put},
};
use chrono::{Duration, NaiveDateTime, Timelike, Utc};
use password_hash::{PasswordHash, PasswordVerifier, SaltString, rand_core::OsRng};
use serde::Deserialize;
use std::collections::HashSet;

pub fn router(db: Db) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/overview", get(overview))
        .route("/api/clients", get(list_clients).post(create_client))
        .route("/api/clients/{id}", put(update_client))
        .route("/api/barbers", get(list_barbers).post(create_barber))
        .route(
            "/api/barbers/{id}",
            put(update_barber).delete(delete_barber),
        )
        .route(
            "/api/barbers/{id}/commissions",
            get(list_commissions).put(update_commission),
        )
        .route("/api/services", get(list_services).post(create_service))
        .route("/api/services/{id}", put(update_service))
        .route(
            "/api/appointments",
            get(list_appointments).post(create_appointment),
        )
        .route("/api/appointments/{id}", put(update_appointment))
        .route("/api/checkouts", post(checkout))
        .route(
            "/api/extra-expenses",
            get(list_extra_expenses).post(create_extra_expense),
        )
        .route("/api/auth/register-barbershop", post(register_barbershop))
        .route("/api/auth/login", post(login))
        .route("/api/auth/forgot-password", post(request_password_reset))
        .route("/api/auth/reset-password", post(reset_password))
        .route("/api/auth/me", get(me))
        .with_state(db)
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
    account_type: String,
}

async fn register_barbershop(
    State(db): State<Db>,
    Json(input): Json<RegisterBarbershop>,
) -> ApiResult<Json<AuthResponse>> {
    let barbershop_name = required_display_text(Some(&input.barbershop_name), "nome da barbearia")?;
    let owner_name = required_display_text(Some(&input.owner_name), "nome do responsavel")?;
    let email = normalize_email(&input.email)?;
    validate_password(&input.password)?;
    ensure_registration_email_available(&db, &email).await?;
    let slug = unique_slug(&db, &barbershop_name).await?;
    let password_hash = hash_password(&input.password)?;

    let mut tx = db.pool.begin().await?;
    let barbershop_id = sqlx::query("insert into barbershops (name, slug) values (?, ?)")
        .bind(&barbershop_name)
        .bind(&slug)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
    let user_id = sqlx::query(
        "insert into users (barbershop_id, name, email, password_hash, role) values (?, ?, ?, ?, 'admin')",
    )
    .bind(barbershop_id)
    .bind(&owner_name)
    .bind(&email)
    .bind(&password_hash)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();
    tx.commit().await?;

    let token = create_session(&db, user_id, "user", barbershop_id, "admin", None).await?;
    let user = auth_identity_from_token(&db, &token).await?;
    Ok(Json(AuthResponse { token, user }))
}

async fn login(
    State(db): State<Db>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let email = normalize_email(&payload.email)?;
    if payload.password.len() < 6 {
        return Err(ApiError::BadRequest("credenciais invalidas".into()));
    }

    match account_type(&payload.account_type)? {
        AccountType::Establishment => {
            if let Some(row) = sqlx::query_as::<_, (i64, String, String, String, String, i64)>(
                "select id, name, email, password_hash, role, barbershop_id from users where lower(email) = ?",
            )
            .bind(&email)
            .fetch_optional(&db.pool)
            .await?
            {
                let (id, _name, _email, password_hash, role, barbershop_id) = row;
                if verify_password(&payload.password, &password_hash) {
                    let token = create_session(&db, id, "user", barbershop_id, &role, None).await?;
                    let user = auth_identity_from_token(&db, &token).await?;
                    return Ok(Json(AuthResponse { token, user }));
                }
            }
        }
        AccountType::Professional => {
            if let Some(row) = sqlx::query_as::<_, (i64, String, String, String, i64)>(
                "select id, name, email, password_hash, barbershop_id
                 from barbers
                 where lower(email) = ? and deleted_at is null and status = 'active'",
            )
            .bind(&email)
            .fetch_optional(&db.pool)
            .await?
            {
                let (id, _name, _email, password_hash, barbershop_id) = row;
                if verify_password(&payload.password, &password_hash) {
                    let token =
                        create_session(&db, id, "barber", barbershop_id, "barber", Some(id)).await?;
                    let user = auth_identity_from_token(&db, &token).await?;
                    return Ok(Json(AuthResponse { token, user }));
                }
            }
        }
    }

    Err(ApiError::Unauthorized)
}

async fn me(State(db): State<Db>, headers: HeaderMap) -> ApiResult<Json<AuthIdentity>> {
    Ok(Json(authenticate(&db, &headers).await?))
}

async fn request_password_reset(
    State(db): State<Db>,
    Json(input): Json<PasswordResetRequest>,
) -> ApiResult<Json<PasswordResetResponse>> {
    let email = normalize_email(&input.email)?;
    let reset_subject =
        find_password_reset_subject(&db, &email, account_type(&input.account_type)?).await?;
    let mut reset_token = None;

    if let Some((subject_id, subject_type)) = reset_subject {
        let token = uuid::Uuid::new_v4().to_string();
        let expires_at = (Utc::now() + Duration::minutes(30))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        sqlx::query(
            "insert into password_reset_tokens (token, subject_id, subject_type, email, expires_at)
             values (?, ?, ?, ?, ?)",
        )
        .bind(&token)
        .bind(subject_id)
        .bind(subject_type)
        .bind(&email)
        .bind(expires_at)
        .execute(&db.pool)
        .await?;
        reset_token = Some(token);
    }

    Ok(Json(password_reset_response(reset_token)))
}

async fn reset_password(
    State(db): State<Db>,
    Json(input): Json<PasswordResetConfirm>,
) -> ApiResult<Json<PasswordResetResponse>> {
    let token = required_display_text(Some(&input.token), "codigo de recuperacao")?;
    validate_password(&input.password)?;

    let reset = sqlx::query_as::<_, (i64, String)>(
        "select subject_id, subject_type
         from password_reset_tokens
         where token = ? and used_at is null and expires_at > current_timestamp",
    )
    .bind(&token)
    .fetch_optional(&db.pool)
    .await?
    .ok_or_else(|| ApiError::BadRequest("codigo de recuperacao invalido ou expirado".into()))?;

    let password_hash = hash_password(&input.password)?;
    let mut tx = db.pool.begin().await?;
    match reset.1.as_str() {
        "user" => {
            sqlx::query("update users set password_hash = ? where id = ?")
                .bind(&password_hash)
                .bind(reset.0)
                .execute(&mut *tx)
                .await?;
        }
        "barber" => {
            sqlx::query("update barbers set password_hash = ? where id = ?")
                .bind(&password_hash)
                .bind(reset.0)
                .execute(&mut *tx)
                .await?;
        }
        _ => {
            return Err(ApiError::BadRequest(
                "codigo de recuperacao invalido".into(),
            ));
        }
    }
    sqlx::query("update password_reset_tokens set used_at = current_timestamp where token = ?")
        .bind(&token)
        .execute(&mut *tx)
        .await?;
    sqlx::query("delete from sessions where subject_id = ? and subject_type = ?")
        .bind(reset.0)
        .bind(&reset.1)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok(Json(PasswordResetResponse {
        message: "senha atualizada com sucesso".into(),
        reset_token: None,
    }))
}

async fn find_password_reset_subject(
    db: &Db,
    email: &str,
    account_type: AccountType,
) -> ApiResult<Option<(i64, &'static str)>> {
    match account_type {
        AccountType::Establishment => {
            if let Some((id,)) =
                sqlx::query_as::<_, (i64,)>("select id from users where lower(email) = ?")
                    .bind(email)
                    .fetch_optional(&db.pool)
                    .await?
            {
                return Ok(Some((id, "user")));
            }
        }
        AccountType::Professional => {
            if let Some((id,)) = sqlx::query_as::<_, (i64,)>(
                "select id from barbers
                 where lower(email) = ? and deleted_at is null and status = 'active'",
            )
            .bind(email)
            .fetch_optional(&db.pool)
            .await?
            {
                return Ok(Some((id, "barber")));
            }
        }
    }

    Ok(None)
}

#[derive(Clone, Copy)]
enum AccountType {
    Establishment,
    Professional,
}

fn account_type(value: &str) -> ApiResult<AccountType> {
    match value.trim() {
        "establishment" => Ok(AccountType::Establishment),
        "professional" => Ok(AccountType::Professional),
        _ => Err(ApiError::BadRequest("tipo de acesso invalido".into())),
    }
}

fn password_reset_response(reset_token: Option<String>) -> PasswordResetResponse {
    let expose_token =
        cfg!(test) || std::env::var("PASSWORD_RESET_EXPOSE_TOKEN").ok().as_deref() == Some("1");
    PasswordResetResponse {
        message: "se o email existir, enviaremos um codigo de recuperacao".into(),
        reset_token: if expose_token { reset_token } else { None },
    }
}

fn normalize_email(email: &str) -> ApiResult<String> {
    let email = required_display_text(Some(email), "email")?.to_lowercase();
    if !email.contains('@') || email.len() < 6 {
        return Err(ApiError::BadRequest("email invalido".into()));
    }
    Ok(email)
}

fn validate_password(password: &str) -> ApiResult<()> {
    if password.len() < 8 {
        return Err(ApiError::BadRequest(
            "senha deve ter pelo menos 8 caracteres".into(),
        ));
    }
    Ok(())
}

async fn unique_slug(db: &Db, name: &str) -> ApiResult<String> {
    let base = slugify(name);
    for suffix in 0..100 {
        let candidate = if suffix == 0 {
            base.clone()
        } else {
            format!("{base}-{suffix}")
        };
        let exists: (i64,) = sqlx::query_as("select count(*) from barbershops where slug = ?")
            .bind(&candidate)
            .fetch_one(&db.pool)
            .await?;
        if exists.0 == 0 {
            return Ok(candidate);
        }
    }
    Err(ApiError::BadRequest(
        "nao foi possivel gerar slug da barbearia".into(),
    ))
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in value.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "barbearia".into()
    } else {
        slug
    }
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(password_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

async fn create_session(
    db: &Db,
    subject_id: i64,
    subject_type: &str,
    barbershop_id: i64,
    role: &str,
    barber_id: Option<i64>,
) -> ApiResult<String> {
    let token = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "insert into sessions (token, subject_id, subject_type, barbershop_id, role, barber_id)
         values (?, ?, ?, ?, ?, ?)",
    )
    .bind(&token)
    .bind(subject_id)
    .bind(subject_type)
    .bind(barbershop_id)
    .bind(role)
    .bind(barber_id)
    .execute(&db.pool)
    .await?;
    Ok(token)
}

async fn authenticate(db: &Db, headers: &HeaderMap) -> ApiResult<AuthIdentity> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(ApiError::Unauthorized)?;
    auth_identity_from_token(db, token).await
}

async fn auth_identity_from_token(db: &Db, token: &str) -> ApiResult<AuthIdentity> {
    sqlx::query_as::<_, AuthIdentity>(
        "select s.subject_id as id,
                coalesce(u.name, b.name) as name,
                coalesce(u.email, b.email) as email,
                s.role,
                s.barbershop_id,
                bs.name as barbershop_name,
                s.barber_id
         from sessions s
         join barbershops bs on bs.id = s.barbershop_id
         left join users u on s.subject_type = 'user' and u.id = s.subject_id
         left join barbers b on s.subject_type = 'barber' and b.id = s.subject_id
         where s.token = ?",
    )
    .bind(token)
    .fetch_optional(&db.pool)
    .await?
    .ok_or(ApiError::Unauthorized)
}

fn require_admin(session: &AuthIdentity) -> ApiResult<()> {
    if matches!(session.role.as_str(), "owner" | "admin") {
        return Ok(());
    }
    Err(ApiError::Forbidden)
}

fn require_barber_self(session: &AuthIdentity, barber_id: i64) -> ApiResult<()> {
    if matches!(session.role.as_str(), "owner" | "admin") {
        return Ok(());
    }
    if session.role == "barber" && session.barber_id == Some(barber_id) {
        return Ok(());
    }
    Err(ApiError::Forbidden)
}

async fn overview(State(db): State<Db>, headers: HeaderMap) -> ApiResult<Json<Overview>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let clients: (i64,) = sqlx::query_as(
        "select count(*) from clients where deleted_at is null and barbershop_id = ?",
    )
    .bind(session.barbershop_id)
    .fetch_one(&db.pool)
    .await?;
    let appointments: (i64,) =
        sqlx::query_as("select count(*) from appointments where barbershop_id = ?")
            .bind(session.barbershop_id)
            .fetch_one(&db.pool)
            .await?;
    let open_appointments: (i64,) = sqlx::query_as(
        "select count(*) from appointments where status != 'completed' and barbershop_id = ?",
    )
    .bind(session.barbershop_id)
    .fetch_one(&db.pool)
    .await?;
    let in_progress_appointments: (i64,) = sqlx::query_as(
        "select count(*) from appointments where status = 'in_chair' and barbershop_id = ?",
    )
    .bind(session.barbershop_id)
    .fetch_one(&db.pool)
    .await?;
    let revenue: (Option<i64>,) =
        sqlx::query_as("select sum(total_cents) from payments where barbershop_id = ?")
            .bind(session.barbershop_id)
            .fetch_one(&db.pool)
            .await?;
    let commissions: (Option<i64>,) = sqlx::query_as(
        "select sum(cast((aps.price_cents * coalesce(bsc.commission_percent, 0)) / 100 as integer))
         from appointment_services aps
         join appointments a on a.id = aps.appointment_id
         left join barber_service_commissions bsc on bsc.barber_id = a.barber_id and bsc.service_id = aps.service_id
         where a.status = 'completed' and a.barbershop_id = ?",
    )
    .bind(session.barbershop_id)
    .fetch_one(&db.pool)
    .await?;
    let extra_expenses: (Option<i64>,) =
        sqlx::query_as("select sum(amount_cents) from extra_expenses where barbershop_id = ?")
            .bind(session.barbershop_id)
            .fetch_one(&db.pool)
            .await?;
    let revenue_cents = revenue.0.unwrap_or(0);
    let commissions_cents = commissions.0.unwrap_or(0);
    let extra_expenses_cents = extra_expenses.0.unwrap_or(0);
    Ok(Json(Overview {
        revenue_cents,
        commissions_cents,
        net_revenue_cents: revenue_cents - commissions_cents,
        extra_expenses_cents,
        profit_cents: financial_profit(revenue_cents, commissions_cents, extra_expenses_cents),
        clients: clients.0,
        appointments: appointments.0,
        open_appointments: open_appointments.0,
        in_progress_appointments: in_progress_appointments.0,
    }))
}

fn financial_profit(revenue_cents: i64, commissions_cents: i64, extra_expenses_cents: i64) -> i64 {
    revenue_cents - commissions_cents - extra_expenses_cents
}

async fn list_clients(State(db): State<Db>, headers: HeaderMap) -> ApiResult<Json<Vec<Client>>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let items = sqlx::query_as::<_, Client>(
        "select id, name, phone, email, document, haircut_frequency, total_spent_cents, visits
         from clients where deleted_at is null and barbershop_id = ? order by name",
    )
    .bind(session.barbershop_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(Json(items))
}

async fn create_client(
    State(db): State<Db>,
    headers: HeaderMap,
    Json(input): Json<CreateClient>,
) -> ApiResult<Json<Client>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let name = required_display_text(Some(&input.name), "nome")?;
    let phone = normalize_mobile_phone(&input.phone).ok_or_else(|| {
        ApiError::BadRequest(
            "telefone deve ser celular com DDD e 9 digitos ou uma sequencia repetida".into(),
        )
    })?;
    let haircut_frequency =
        required_display_text(input.haircut_frequency.as_deref(), "frequencia de corte")?;
    let email = optional_display_text(input.email.as_deref(), "email")?;
    let document = optional_document(input.document.as_deref())?;
    ensure_client_contact_available(
        &db,
        session.barbershop_id,
        &phone,
        document.as_deref(),
        None,
    )
    .await?;
    let id = sqlx::query(
        "insert into clients (barbershop_id, name, phone, email, document, haircut_frequency) values (?, ?, ?, ?, ?, ?)",
    )
    .bind(session.barbershop_id)
    .bind(name)
    .bind(phone)
    .bind(email)
    .bind(document)
    .bind(haircut_frequency)
    .execute(&db.pool)
    .await?
    .last_insert_rowid();
    let item = sqlx::query_as::<_, Client>(
        "select id, name, phone, email, document, haircut_frequency, total_spent_cents, visits from clients where id = ? and barbershop_id = ?",
    )
    .bind(id)
    .bind(session.barbershop_id)
    .fetch_one(&db.pool)
    .await?;
    Ok(Json(item))
}

async fn update_client(
    State(db): State<Db>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<UpdateClient>,
) -> ApiResult<Json<Client>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let name = required_display_text(Some(&input.name), "nome")?;
    let phone = normalize_mobile_phone(&input.phone).ok_or_else(|| {
        ApiError::BadRequest(
            "telefone deve ser celular com DDD e 9 digitos ou uma sequencia repetida".into(),
        )
    })?;
    let haircut_frequency =
        required_display_text(input.haircut_frequency.as_deref(), "frequencia de corte")?;
    let email = optional_display_text(input.email.as_deref(), "email")?;
    let document = optional_document(input.document.as_deref())?;

    let client_exists: (i64,) = sqlx::query_as(
        "select exists(select 1 from clients where id = ? and barbershop_id = ? and deleted_at is null)",
    )
    .bind(id)
    .bind(session.barbershop_id)
    .fetch_one(&db.pool)
    .await?;
    if client_exists.0 == 0 {
        return Err(ApiError::NotFound);
    }

    ensure_client_contact_available(
        &db,
        session.barbershop_id,
        &phone,
        document.as_deref(),
        Some(id),
    )
    .await?;

    sqlx::query(
        "update clients set name = ?, phone = ?, email = ?, document = ?, haircut_frequency = ? where id = ? and barbershop_id = ? and deleted_at is null",
    )
    .bind(name)
    .bind(phone)
    .bind(email)
    .bind(document)
    .bind(haircut_frequency)
    .bind(id)
    .bind(session.barbershop_id)
    .execute(&db.pool)
    .await?;

    let item = sqlx::query_as::<_, Client>(
        "select id, name, phone, email, document, haircut_frequency, total_spent_cents, visits from clients where id = ? and barbershop_id = ?",
    )
    .bind(id)
    .bind(session.barbershop_id)
    .fetch_one(&db.pool)
    .await?;

    Ok(Json(item))
}

async fn ensure_client_contact_available(
    db: &Db,
    barbershop_id: i64,
    phone: &str,
    document: Option<&str>,
    except_client_id: Option<i64>,
) -> ApiResult<()> {
    let phone_exists: (i64,) = if let Some(except_id) = except_client_id {
        sqlx::query_as(
            "select exists(select 1 from clients where barbershop_id = ? and phone = ? and id != ? and deleted_at is null)",
        )
        .bind(barbershop_id)
        .bind(phone)
        .bind(except_id)
        .fetch_one(&db.pool)
        .await?
    } else {
        sqlx::query_as(
            "select exists(select 1 from clients where barbershop_id = ? and phone = ? and deleted_at is null)",
        )
        .bind(barbershop_id)
        .bind(phone)
        .fetch_one(&db.pool)
        .await?
    };
    if phone_exists.0 != 0 {
        return Err(ApiError::BadRequest(
            "telefone ja cadastrado para outro cliente".into(),
        ));
    }

    if let Some(document) = document {
        let document_exists: (i64,) = if let Some(except_id) = except_client_id {
            sqlx::query_as(
                "select exists(select 1 from clients where barbershop_id = ? and document = ? and id != ? and deleted_at is null)",
            )
            .bind(barbershop_id)
            .bind(document)
            .bind(except_id)
            .fetch_one(&db.pool)
            .await?
        } else {
            sqlx::query_as(
                "select exists(select 1 from clients where barbershop_id = ? and document = ? and deleted_at is null)",
            )
            .bind(barbershop_id)
            .bind(document)
            .fetch_one(&db.pool)
            .await?
        };
        if document_exists.0 != 0 {
            return Err(ApiError::BadRequest(
                "CPF ja cadastrado para outro cliente".into(),
            ));
        }
    }

    Ok(())
}

fn normalize_mobile_phone(phone: &str) -> Option<String> {
    let digits: String = phone.chars().filter(char::is_ascii_digit).collect();
    if is_repeated_digit_placeholder(&digits) {
        return Some(digits);
    }
    let area_code = digits.get(0..2)?.parse::<u8>().ok()?;
    let is_valid = digits.len() == 11
        && (11..=99).contains(&area_code)
        && digits.as_bytes().get(2) == Some(&b'9');
    is_valid.then_some(digits)
}

fn is_repeated_digit_placeholder(digits: &str) -> bool {
    digits.len() >= 3
        && digits
            .as_bytes()
            .first()
            .is_some_and(|first| digits.as_bytes().iter().all(|digit| digit == first))
}

fn optional_document(document: Option<&str>) -> ApiResult<Option<String>> {
    let digits: String = document
        .unwrap_or_default()
        .chars()
        .filter(char::is_ascii_digit)
        .collect();
    if digits.is_empty() {
        Ok(None)
    } else if is_valid_cpf(&digits) {
        Ok(Some(digits))
    } else {
        Err(ApiError::BadRequest("CPF invalido".into()))
    }
}

fn required_document(document: Option<&str>) -> ApiResult<String> {
    optional_document(document)?
        .ok_or_else(|| ApiError::BadRequest("CPF do profissional e obrigatorio".into()))
}

fn is_valid_cpf(digits: &str) -> bool {
    if digits.len() != 11 || is_repeated_digit_placeholder(digits) {
        return false;
    }

    let bytes = digits.as_bytes();
    cpf_check_digit(bytes, 9) == bytes[9] - b'0' && cpf_check_digit(bytes, 10) == bytes[10] - b'0'
}

fn cpf_check_digit(bytes: &[u8], len: usize) -> u8 {
    let sum: u32 = bytes
        .iter()
        .take(len)
        .enumerate()
        .map(|(index, digit)| u32::from(digit - b'0') * ((len + 1 - index) as u32))
        .sum();
    let remainder = sum % 11;
    if remainder < 2 {
        0
    } else {
        (11 - remainder) as u8
    }
}

fn required_display_text(value: Option<&str>, field: &str) -> ApiResult<String> {
    optional_display_text(value, field)?
        .ok_or_else(|| ApiError::BadRequest(format!("{field} e obrigatorio")))
}

fn optional_display_text(value: Option<&str>, field: &str) -> ApiResult<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.len() > 180 {
        return Err(ApiError::BadRequest(format!("{field} muito longo")));
    }
    if trimmed
        .chars()
        .any(|ch| ch.is_control() || matches!(ch, '<' | '>'))
    {
        return Err(ApiError::BadRequest(format!(
            "{field} contem caracteres invalidos"
        )));
    }
    Ok(Some(trimmed.to_string()))
}

fn validate_non_negative_cents(value: i64, field: &str) -> ApiResult<i64> {
    if value < 0 {
        return Err(ApiError::BadRequest(format!(
            "{field} nao pode ser negativo"
        )));
    }
    Ok(value)
}

fn validate_positive_cents(value: i64, field: &str) -> ApiResult<i64> {
    if value <= 0 {
        return Err(ApiError::BadRequest(format!(
            "{field} deve ser maior que zero"
        )));
    }
    Ok(value)
}

fn validate_positive_id(id: i64, field: &str) -> ApiResult<i64> {
    if id <= 0 {
        return Err(ApiError::BadRequest(format!("{field} invalido")));
    }
    Ok(id)
}

fn validate_service_ids(service_ids: &[i64]) -> ApiResult<Vec<i64>> {
    if service_ids.is_empty() {
        return Err(ApiError::BadRequest("selecione ao menos um servico".into()));
    }
    let mut seen = HashSet::with_capacity(service_ids.len());
    for service_id in service_ids {
        let service_id = validate_positive_id(*service_id, "servico")?;
        if !seen.insert(service_id) {
            return Err(ApiError::BadRequest(
                "servicos duplicados no agendamento".into(),
            ));
        }
    }
    Ok(service_ids.to_vec())
}

fn validate_payment_method(method: &str) -> ApiResult<String> {
    let method = method.trim().to_lowercase();
    if matches!(method.as_str(), "pix" | "cash" | "debit" | "credit") {
        return Ok(method);
    }
    Err(ApiError::BadRequest("forma de pagamento invalida".into()))
}

fn checkout_payment_lines(input: &CheckoutRequest) -> ApiResult<Vec<CheckoutPaymentLine>> {
    if input.payments.is_empty() {
        let method = validate_payment_method(&input.payment_method)?;
        let amount_cents = validate_non_negative_cents(input.paid_cents, "valor pago")?;
        return Ok(vec![CheckoutPaymentLine {
            method,
            amount_cents,
        }]);
    }

    let mut lines = Vec::with_capacity(input.payments.len());
    for payment in &input.payments {
        lines.push(CheckoutPaymentLine {
            method: validate_payment_method(&payment.method)?,
            amount_cents: validate_positive_cents(payment.amount_cents, "valor do pagamento")?,
        });
    }
    Ok(lines)
}

fn validate_appointment_start(starts_at: &str) -> ApiResult<String> {
    let starts_at = required_display_text(Some(starts_at), "horario")?;
    let parsed = NaiveDateTime::parse_from_str(&starts_at, "%Y-%m-%dT%H:%M")
        .map_err(|_| ApiError::BadRequest("horario do agendamento invalido".into()))?;
    let hour = parsed.hour();
    if !(9..22).contains(&hour) {
        return Err(ApiError::BadRequest(
            "agendamento deve estar entre 09:00 e 21:59".into(),
        ));
    }
    Ok(starts_at)
}

async fn list_barbers(State(db): State<Db>, headers: HeaderMap) -> ApiResult<Json<Vec<Barber>>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let items = sqlx::query_as::<_, Barber>(
        "select id, name, document, email, specialty, status, monthly_commission_cents, monthly_tips_cents, completed_services
         from barbers where deleted_at is null and barbershop_id = ? order by name",
    )
    .bind(session.barbershop_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(Json(items))
}

async fn create_barber(
    State(db): State<Db>,
    headers: HeaderMap,
    Json(input): Json<CreateBarber>,
) -> ApiResult<Json<Barber>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let name = required_display_text(Some(&input.name), "nome do profissional")?;
    let document = required_document(Some(&input.document))?;
    let email = required_display_text(Some(&input.email), "email do profissional")?.to_lowercase();
    let specialty =
        optional_display_text(input.specialty.as_deref(), "especialidade")?.unwrap_or_default();

    if !email.contains('@') || email.len() < 6 {
        return Err(ApiError::BadRequest(
            "email do profissional e obrigatorio".into(),
        ));
    }
    ensure_barber_identity_available(&db, &email, &document, None).await?;
    validate_password(&input.password)?;
    let password_hash = hash_password(&input.password)?;
    let id = sqlx::query(
        "insert into barbers (barbershop_id, name, document, email, password_hash, specialty) values (?, ?, ?, ?, ?, ?)",
    )
    .bind(session.barbershop_id)
    .bind(name)
    .bind(document)
    .bind(email)
    .bind(password_hash)
    .bind(specialty)
    .execute(&db.pool)
    .await?
    .last_insert_rowid();
    let services: Vec<(i64,)> = sqlx::query_as("select id from services where barbershop_id = ?")
        .bind(session.barbershop_id)
        .fetch_all(&db.pool)
        .await?;
    for (service_id,) in services {
        sqlx::query(
            "insert or ignore into barber_service_commissions (barber_id, service_id, commission_percent) values (?, ?, 30)",
        )
        .bind(id)
        .bind(service_id)
        .execute(&db.pool)
        .await?;
    }
    let item = sqlx::query_as::<_, Barber>(
        "select id, name, document, email, specialty, status, monthly_commission_cents, monthly_tips_cents, completed_services from barbers where id = ? and barbershop_id = ?",
    )
    .bind(id)
    .bind(session.barbershop_id)
    .fetch_one(&db.pool)
    .await?;
    Ok(Json(item))
}

async fn update_barber(
    State(db): State<Db>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<UpdateBarber>,
) -> ApiResult<Json<Barber>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let name = required_display_text(Some(&input.name), "nome do profissional")?;
    let document = required_document(Some(&input.document))?;
    let email = required_display_text(Some(&input.email), "email do profissional")?.to_lowercase();
    let specialty =
        optional_display_text(input.specialty.as_deref(), "especialidade")?.unwrap_or_default();
    let status = input.status.unwrap_or_else(|| "active".to_string());
    let status = status.trim();

    if !email.contains('@') || email.len() < 6 {
        return Err(ApiError::BadRequest(
            "email do profissional e obrigatorio".into(),
        ));
    }
    ensure_barber_identity_available(&db, &email, &document, Some(id)).await?;
    if !matches!(status, "active" | "inactive") {
        return Err(ApiError::BadRequest(
            "status do profissional invalido".into(),
        ));
    }

    if let Some(password) = optional_password_update(input.password.as_deref()) {
        validate_password(password)?;
        let password_hash = hash_password(password)?;
        sqlx::query(
            "update barbers set name = ?, document = ?, email = ?, password_hash = ?, specialty = ?, status = ? where id = ? and barbershop_id = ? and deleted_at is null",
        )
        .bind(name)
        .bind(document)
        .bind(email)
        .bind(password_hash)
        .bind(specialty)
        .bind(status)
        .bind(id)
        .bind(session.barbershop_id)
        .execute(&db.pool)
        .await?;
    } else {
        sqlx::query("update barbers set name = ?, document = ?, email = ?, specialty = ?, status = ? where id = ? and barbershop_id = ? and deleted_at is null")
            .bind(name)
            .bind(document)
            .bind(email)
            .bind(specialty)
            .bind(status)
            .bind(id)
            .bind(session.barbershop_id)
            .execute(&db.pool)
            .await?;
    }

    let item = sqlx::query_as::<_, Barber>(
        "select id, name, document, email, specialty, status, monthly_commission_cents, monthly_tips_cents, completed_services
         from barbers where id = ? and barbershop_id = ? and deleted_at is null",
    )
    .bind(id)
    .bind(session.barbershop_id)
    .fetch_optional(&db.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    Ok(Json(item))
}

async fn ensure_barber_identity_available(
    db: &Db,
    email: &str,
    document: &str,
    except_barber_id: Option<i64>,
) -> ApiResult<()> {
    let existing_document: Option<(String,)> = if let Some(except_barber_id) = except_barber_id {
        sqlx::query_as(
            "select lower(email) from barbers where document = ? and id != ? and deleted_at is null limit 1",
        )
        .bind(document)
        .bind(except_barber_id)
        .fetch_optional(&db.pool)
        .await?
    } else {
        sqlx::query_as(
            "select lower(email) from barbers where document = ? and deleted_at is null limit 1",
        )
        .bind(document)
        .fetch_optional(&db.pool)
        .await?
    };
    if let Some((existing_email,)) = existing_document {
        if existing_email == email {
            return Err(ApiError::BadRequest(
                "CPF ja cadastrado para outro profissional".into(),
            ));
        }
        return Err(ApiError::BadRequest(
            "CPF ja vinculado a outro email".into(),
        ));
    }

    ensure_barber_email_available(db, email, except_barber_id).await
}

async fn ensure_barber_email_available(
    db: &Db,
    email: &str,
    except_barber_id: Option<i64>,
) -> ApiResult<()> {
    let barber_exists = if let Some(except_barber_id) = except_barber_id {
        sqlx::query_as::<_, (i64,)>(
            "select exists(select 1 from barbers where lower(email) = ? and id != ? and deleted_at is null)",
        )
        .bind(email)
        .bind(except_barber_id)
        .fetch_one(&db.pool)
        .await?
    } else {
        sqlx::query_as::<_, (i64,)>(
            "select exists(select 1 from barbers where lower(email) = ? and deleted_at is null)",
        )
        .bind(email)
        .fetch_one(&db.pool)
        .await?
    };
    if barber_exists.0 != 0 {
        return Err(ApiError::BadRequest(
            "email do profissional ja cadastrado".into(),
        ));
    }

    let user_exists: (i64,) =
        sqlx::query_as("select exists(select 1 from users where lower(email) = ?)")
            .bind(email)
            .fetch_one(&db.pool)
            .await?;
    if user_exists.0 != 0 {
        return Err(ApiError::BadRequest(
            "email ja cadastrado para acesso ao sistema".into(),
        ));
    }

    Ok(())
}

async fn ensure_registration_email_available(db: &Db, email: &str) -> ApiResult<()> {
    let user_exists: (i64,) =
        sqlx::query_as("select exists(select 1 from users where lower(email) = ?)")
            .bind(email)
            .fetch_one(&db.pool)
            .await?;
    if user_exists.0 != 0 {
        return Err(ApiError::BadRequest(
            "email ja cadastrado para acesso ao sistema".into(),
        ));
    }

    let barber_exists: (i64,) = sqlx::query_as(
        "select exists(select 1 from barbers where lower(email) = ? and deleted_at is null)",
    )
    .bind(email)
    .fetch_one(&db.pool)
    .await?;
    if barber_exists.0 != 0 {
        return Err(ApiError::BadRequest(
            "email ja cadastrado para acesso ao sistema".into(),
        ));
    }

    Ok(())
}

async fn delete_barber(
    State(db): State<Db>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let result = sqlx::query("update barbers set deleted_at = current_timestamp, status = 'inactive' where id = ? and barbershop_id = ? and deleted_at is null")
        .bind(id)
        .bind(session.barbershop_id)
        .execute(&db.pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(Json(serde_json::json!({ "deleted": true })))
}

fn optional_password_update(password: Option<&str>) -> Option<&str> {
    let password = password?.trim();
    (!password.is_empty()).then_some(password)
}

async fn list_services(State(db): State<Db>, headers: HeaderMap) -> ApiResult<Json<Vec<Service>>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let items = sqlx::query_as::<_, Service>(
        "select id, name, description, duration_minutes, price_cents, category, active
         from services where barbershop_id = ?
         order by active desc, price_cents desc",
    )
    .bind(session.barbershop_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(Json(items))
}

async fn create_service(
    State(db): State<Db>,
    headers: HeaderMap,
    Json(input): Json<CreateService>,
) -> ApiResult<Json<Service>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let name = required_display_text(Some(&input.name), "nome do servico")?;
    let description =
        optional_display_text(input.description.as_deref(), "descricao")?.unwrap_or_default();
    let category = optional_display_text(input.category.as_deref(), "categoria")?
        .unwrap_or_else(|| "geral".to_string());
    let price_cents = validate_non_negative_cents(input.price_cents, "preco")?;
    if input.duration_minutes <= 0 {
        return Err(ApiError::BadRequest("servico invalido".into()));
    }
    let id = sqlx::query(
        "insert into services (barbershop_id, name, description, duration_minutes, price_cents, category, active) values (?, ?, ?, ?, ?, ?, 1)",
    )
    .bind(session.barbershop_id)
    .bind(name)
    .bind(description)
    .bind(input.duration_minutes)
    .bind(price_cents)
    .bind(category)
    .execute(&db.pool)
    .await?
    .last_insert_rowid();
    let barbers: Vec<(i64,)> = sqlx::query_as("select id from barbers where barbershop_id = ?")
        .bind(session.barbershop_id)
        .fetch_all(&db.pool)
        .await?;
    for (barber_id,) in barbers {
        sqlx::query(
            "insert or ignore into barber_service_commissions (barber_id, service_id, commission_percent) values (?, ?, 30)",
        )
        .bind(barber_id)
        .bind(id)
        .execute(&db.pool)
        .await?;
    }
    let item = sqlx::query_as::<_, Service>(
        "select id, name, description, duration_minutes, price_cents, category, active from services where id = ? and barbershop_id = ?",
    )
    .bind(id)
    .bind(session.barbershop_id)
    .fetch_one(&db.pool)
    .await?;
    Ok(Json(item))
}

async fn update_service(
    State(db): State<Db>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<UpdateService>,
) -> ApiResult<Json<Service>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let id = validate_positive_id(id, "servico")?;
    let name = required_display_text(Some(&input.name), "nome do servico")?;
    let description =
        optional_display_text(input.description.as_deref(), "descricao")?.unwrap_or_default();
    let category = optional_display_text(input.category.as_deref(), "categoria")?
        .unwrap_or_else(|| "geral".to_string());
    let price_cents = validate_non_negative_cents(input.price_cents, "preco")?;
    if input.duration_minutes <= 0 {
        return Err(ApiError::BadRequest("servico invalido".into()));
    }
    let active = input.active.unwrap_or(true);
    let result = sqlx::query(
        "update services
         set name = ?, description = ?, duration_minutes = ?, price_cents = ?, category = ?, active = ?
         where id = ? and barbershop_id = ?",
    )
    .bind(name)
    .bind(description)
    .bind(input.duration_minutes)
    .bind(price_cents)
    .bind(category)
    .bind(active)
    .bind(id)
    .bind(session.barbershop_id)
    .execute(&db.pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    let item = sqlx::query_as::<_, Service>(
        "select id, name, description, duration_minutes, price_cents, category, active from services where id = ? and barbershop_id = ?",
    )
    .bind(id)
    .bind(session.barbershop_id)
    .fetch_one(&db.pool)
    .await?;
    Ok(Json(item))
}

fn hash_password(password: &str) -> ApiResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| ApiError::BadRequest("nao foi possivel proteger a senha".into()))
}

async fn list_appointments(
    State(db): State<Db>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<Appointment>>> {
    let session = authenticate(&db, &headers).await?;
    let items = if session.role == "barber" {
        sqlx::query_as::<_, Appointment>(
            "select a.id, a.client_id, c.name as client_name, a.barber_id, b.name as barber_name, a.starts_at, a.status, a.total_cents,
             group_concat(s.name, ' + ') as services,
             group_concat(s.id, ',') as service_ids
             from appointments a
             join clients c on c.id = a.client_id
             join barbers b on b.id = a.barber_id
             join appointment_services aps on aps.appointment_id = a.id
             join services s on s.id = aps.service_id
             where a.barbershop_id = ? and a.barber_id = ?
             group by a.id
             order by a.starts_at",
        )
        .bind(session.barbershop_id)
        .bind(session.barber_id.ok_or(ApiError::Forbidden)?)
        .fetch_all(&db.pool)
        .await?
    } else {
        require_admin(&session)?;
        sqlx::query_as::<_, Appointment>(
        "select a.id, a.client_id, c.name as client_name, a.barber_id, b.name as barber_name, a.starts_at, a.status, a.total_cents,
         group_concat(s.name, ' + ') as services,
         group_concat(s.id, ',') as service_ids
         from appointments a
         join clients c on c.id = a.client_id
         join barbers b on b.id = a.barber_id
         join appointment_services aps on aps.appointment_id = a.id
         join services s on s.id = aps.service_id
         where a.barbershop_id = ?
         group by a.id
         order by a.starts_at",
        )
        .bind(session.barbershop_id)
        .fetch_all(&db.pool)
        .await?
    };
    Ok(Json(items))
}

async fn create_appointment(
    State(db): State<Db>,
    headers: HeaderMap,
    Json(input): Json<CreateAppointment>,
) -> ApiResult<Json<Appointment>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let client_id = validate_positive_id(input.client_id, "cliente")?;
    let barber_id = validate_positive_id(input.barber_id, "profissional")?;
    let service_ids = validate_service_ids(&input.service_ids)?;
    let starts_at = validate_appointment_start(&input.starts_at)?;
    let mut tx = db.pool.begin().await?;
    ensure_exists(&mut tx, "clients", client_id, session.barbershop_id).await?;
    ensure_exists(&mut tx, "barbers", barber_id, session.barbershop_id).await?;
    let mut total = 0;
    for service_id in &service_ids {
        let price: Option<(i64,)> = sqlx::query_as(
            "select price_cents from services where id = ? and active = 1 and barbershop_id = ?",
        )
        .bind(*service_id)
        .bind(session.barbershop_id)
        .fetch_optional(&mut *tx)
        .await?;
        total += price.ok_or(ApiError::NotFound)?.0;
    }
    let id = sqlx::query(
        "insert into appointments (barbershop_id, client_id, barber_id, starts_at, status, total_cents) values (?, ?, ?, ?, 'scheduled', ?)",
    )
    .bind(session.barbershop_id)
    .bind(client_id)
    .bind(barber_id)
    .bind(starts_at)
    .bind(total)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();
    for service_id in service_ids {
        let price: (i64,) =
            sqlx::query_as("select price_cents from services where id = ? and barbershop_id = ?")
                .bind(service_id)
                .bind(session.barbershop_id)
                .fetch_one(&mut *tx)
                .await?;
        sqlx::query(
            "insert into appointment_services (appointment_id, service_id, price_cents) values (?, ?, ?)",
        )
        .bind(id)
        .bind(service_id)
        .bind(price.0)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    let created = sqlx::query_as::<_, Appointment>(appointment_select_by_id_sql())
        .bind(id)
        .bind(session.barbershop_id)
        .fetch_one(&db.pool)
        .await?;
    Ok(Json(created))
}

async fn update_appointment(
    State(db): State<Db>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<UpdateAppointment>,
) -> ApiResult<Json<Appointment>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let id = validate_positive_id(id, "agendamento")?;
    let client_id = validate_positive_id(input.client_id, "cliente")?;
    let barber_id = validate_positive_id(input.barber_id, "profissional")?;
    let service_ids = validate_service_ids(&input.service_ids)?;
    let starts_at = validate_appointment_start(&input.starts_at)?;
    let status = validate_editable_appointment_status(&input.status)?;

    let mut tx = db.pool.begin().await?;
    let existing: Option<(String,)> =
        sqlx::query_as("select status from appointments where id = ? and barbershop_id = ?")
            .bind(id)
            .bind(session.barbershop_id)
            .fetch_optional(&mut *tx)
            .await?;
    let (existing_status,) = existing.ok_or(ApiError::NotFound)?;
    if existing_status == "completed" {
        return Err(ApiError::BadRequest(
            "agendamento ja finalizado nao pode ser editado".into(),
        ));
    }

    ensure_exists(&mut tx, "clients", client_id, session.barbershop_id).await?;
    ensure_exists(&mut tx, "barbers", barber_id, session.barbershop_id).await?;
    let mut total = 0;
    for service_id in &service_ids {
        let price: Option<(i64,)> = sqlx::query_as(
            "select price_cents from services where id = ? and active = 1 and barbershop_id = ?",
        )
        .bind(*service_id)
        .bind(session.barbershop_id)
        .fetch_optional(&mut *tx)
        .await?;
        total += price.ok_or(ApiError::NotFound)?.0;
    }

    sqlx::query(
        "update appointments
         set client_id = ?, barber_id = ?, starts_at = ?, status = ?, total_cents = ?
         where id = ? and barbershop_id = ?",
    )
    .bind(client_id)
    .bind(barber_id)
    .bind(starts_at)
    .bind(status)
    .bind(total)
    .bind(id)
    .bind(session.barbershop_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("delete from appointment_services where appointment_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for service_id in service_ids {
        let price: (i64,) =
            sqlx::query_as("select price_cents from services where id = ? and barbershop_id = ?")
                .bind(service_id)
                .bind(session.barbershop_id)
                .fetch_one(&mut *tx)
                .await?;
        sqlx::query(
            "insert into appointment_services (appointment_id, service_id, price_cents) values (?, ?, ?)",
        )
        .bind(id)
        .bind(service_id)
        .bind(price.0)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    let updated = sqlx::query_as::<_, Appointment>(appointment_select_by_id_sql())
        .bind(id)
        .bind(session.barbershop_id)
        .fetch_one(&db.pool)
        .await?;
    Ok(Json(updated))
}

fn validate_editable_appointment_status(status: &str) -> ApiResult<&str> {
    let status = status.trim();
    match status {
        "scheduled" | "in_chair" | "cancelled" => Ok(status),
        "completed" => Err(ApiError::BadRequest(
            "agendamento finalizado somente pelo fechamento de comanda".into(),
        )),
        _ => Err(ApiError::BadRequest(
            "status do agendamento invalido".into(),
        )),
    }
}

fn appointment_select_by_id_sql() -> &'static str {
    "select a.id, a.client_id, c.name as client_name, a.barber_id, b.name as barber_name, a.starts_at, a.status, a.total_cents,
     group_concat(s.name, ' + ') as services,
     group_concat(s.id, ',') as service_ids
     from appointments a
     join clients c on c.id = a.client_id
     join barbers b on b.id = a.barber_id
     join appointment_services aps on aps.appointment_id = a.id
     join services s on s.id = aps.service_id
     where a.id = ? and a.barbershop_id = ?
     group by a.id"
}

async fn ensure_exists(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    table: &str,
    id: i64,
    barbershop_id: i64,
) -> ApiResult<()> {
    let query = match table {
        "clients" => {
            "select exists(select 1 from clients where id = ? and barbershop_id = ? and deleted_at is null)"
        }
        "barbers" => {
            "select exists(select 1 from barbers where id = ? and barbershop_id = ? and deleted_at is null)"
        }
        "services" => {
            "select exists(select 1 from services where id = ? and barbershop_id = ? and active = 1)"
        }
        _ => return Err(ApiError::BadRequest("relacao invalida".into())),
    };
    let exists: (i64,) = sqlx::query_as(query)
        .bind(id)
        .bind(barbershop_id)
        .fetch_one(&mut **tx)
        .await?;
    if exists.0 == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(())
}

async fn checkout(
    State(db): State<Db>,
    headers: HeaderMap,
    Json(input): Json<CheckoutRequest>,
) -> ApiResult<Json<CheckoutResponse>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let appointment_id = validate_positive_id(input.appointment_id, "agendamento")?;
    let discount_cents = validate_non_negative_cents(input.discount_cents, "desconto")?;
    let payment_lines = checkout_payment_lines(&input)?;
    let split_checkout = !input.payments.is_empty();
    let paid_cents: i64 = payment_lines
        .iter()
        .map(|payment| payment.amount_cents)
        .sum();
    let legacy_tip_cents = if split_checkout {
        0
    } else {
        validate_non_negative_cents(input.tip_cents, "gorjeta")?
    };
    let primary_payment_method = payment_lines
        .first()
        .map(|payment| payment.method.clone())
        .ok_or_else(|| ApiError::BadRequest("informe ao menos uma forma de pagamento".into()))?;
    let mut tx = db.pool.begin().await?;
    let appointment: Option<(i64, String, i64)> = sqlx::query_as(
        "select total_cents, status, barber_id from appointments where id = ? and barbershop_id = ?",
    )
    .bind(appointment_id)
    .bind(session.barbershop_id)
    .fetch_optional(&mut *tx)
    .await?;
    let (subtotal, status, barber_id) = appointment.ok_or(ApiError::NotFound)?;
    if !matches!(status.as_str(), "scheduled" | "waiting" | "in_chair") {
        let message = match status.as_str() {
            "completed" => "agendamento ja finalizado",
            "cancelled" => "agendamento cancelado nao pode ser fechado",
            _ => "status do agendamento nao permite checkout",
        };
        return Err(ApiError::BadRequest(message.into()));
    }
    let amount_due = subtotal - discount_cents;
    if amount_due < 0 {
        return Err(ApiError::BadRequest("desconto maior que subtotal".into()));
    }
    if split_checkout && paid_cents < amount_due {
        return Err(ApiError::BadRequest("valor pago insuficiente".into()));
    }
    let tip_cents = if split_checkout {
        paid_cents - amount_due
    } else {
        legacy_tip_cents
    };
    let total = amount_due + tip_cents;
    if paid_cents < total {
        return Err(ApiError::BadRequest("valor pago insuficiente".into()));
    }
    let change = if split_checkout {
        0
    } else {
        paid_cents - total
    };
    let updated = sqlx::query(
        "update appointments
         set status = 'completed'
         where id = ? and barbershop_id = ? and status in ('scheduled', 'waiting', 'in_chair')",
    )
    .bind(appointment_id)
    .bind(session.barbershop_id)
    .execute(&mut *tx)
    .await?;
    if updated.rows_affected() != 1 {
        return Err(ApiError::BadRequest(
            "agendamento ja foi fechado por outra operacao".into(),
        ));
    }
    let payment_id = sqlx::query(
        "insert into payments (barbershop_id, appointment_id, method, subtotal_cents, discount_cents, tip_cents, paid_cents, change_cents, total_cents)
         values (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(session.barbershop_id)
    .bind(appointment_id)
    .bind(&primary_payment_method)
    .bind(subtotal)
    .bind(discount_cents)
    .bind(tip_cents)
    .bind(paid_cents)
    .bind(change)
    .bind(total)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();
    for payment in &payment_lines {
        sqlx::query(
            "insert into payment_splits (payment_id, method, amount_cents) values (?, ?, ?)",
        )
        .bind(payment_id)
        .bind(&payment.method)
        .bind(payment.amount_cents)
        .execute(&mut *tx)
        .await?;
    }
    if tip_cents > 0 {
        sqlx::query(
            "update barbers set monthly_tips_cents = monthly_tips_cents + ? where id = ? and barbershop_id = ?",
        )
        .bind(tip_cents)
        .bind(barber_id)
        .bind(session.barbershop_id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(Json(CheckoutResponse {
        appointment_id,
        subtotal_cents: subtotal,
        discount_cents,
        tip_cents,
        total_cents: total,
        paid_cents,
        change_cents: change,
        payment_method: primary_payment_method,
        payments: payment_lines,
    }))
}

async fn list_extra_expenses(
    State(db): State<Db>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<ExtraExpense>>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let items = sqlx::query_as::<_, ExtraExpense>(
        "select id, description, amount_cents, created_at from extra_expenses where barbershop_id = ? order by created_at desc, id desc",
    )
    .bind(session.barbershop_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(Json(items))
}

async fn create_extra_expense(
    State(db): State<Db>,
    headers: HeaderMap,
    Json(input): Json<CreateExtraExpense>,
) -> ApiResult<Json<ExtraExpense>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let description = required_display_text(Some(&input.description), "gasto")?;
    let amount_cents = validate_positive_cents(input.amount_cents, "gasto")?;
    let id = sqlx::query(
        "insert into extra_expenses (barbershop_id, description, amount_cents) values (?, ?, ?)",
    )
    .bind(session.barbershop_id)
    .bind(description)
    .bind(amount_cents)
    .execute(&db.pool)
    .await?
    .last_insert_rowid();
    let item = sqlx::query_as::<_, ExtraExpense>(
        "select id, description, amount_cents, created_at from extra_expenses where id = ? and barbershop_id = ?",
    )
    .bind(id)
    .bind(session.barbershop_id)
    .fetch_one(&db.pool)
    .await?;
    Ok(Json(item))
}

async fn list_commissions(
    State(db): State<Db>,
    headers: HeaderMap,
    Path(barber_id): Path<i64>,
) -> ApiResult<Json<Vec<Commission>>> {
    let session = authenticate(&db, &headers).await?;
    require_barber_self(&session, barber_id)?;
    let items = sqlx::query_as::<_, Commission>(
        "select c.barber_id, c.service_id, s.name as service_name, s.price_cents, c.commission_percent,
         cast((s.price_cents * c.commission_percent) / 100 as integer) as estimated_return_cents
         from barber_service_commissions c
         join services s on s.id = c.service_id
         join barbers b on b.id = c.barber_id
         where c.barber_id = ? and b.barbershop_id = ?
         order by s.price_cents desc",
    )
    .bind(barber_id)
    .bind(session.barbershop_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(Json(items))
}

async fn update_commission(
    State(db): State<Db>,
    headers: HeaderMap,
    Path(barber_id): Path<i64>,
    Json(input): Json<CommissionInput>,
) -> ApiResult<Json<Vec<Commission>>> {
    let session = authenticate(&db, &headers).await?;
    require_admin(&session)?;
    let barber_id = validate_positive_id(barber_id, "profissional")?;
    let service_id = validate_positive_id(input.service_id, "servico")?;
    if !(0..=100).contains(&input.commission_percent) {
        return Err(ApiError::BadRequest(
            "comissao deve estar entre 0 e 100".into(),
        ));
    }
    let mut tx = db.pool.begin().await?;
    ensure_exists(&mut tx, "barbers", barber_id, session.barbershop_id).await?;
    ensure_exists(&mut tx, "services", service_id, session.barbershop_id).await?;
    sqlx::query(
        "insert into barber_service_commissions (barber_id, service_id, commission_percent) values (?, ?, ?)
         on conflict(barber_id, service_id) do update set commission_percent = excluded.commission_percent",
    )
    .bind(barber_id)
    .bind(service_id)
    .bind(input.commission_percent)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    list_commissions(State(db), headers, Path(barber_id)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Db, migrate};
    use axum::{Json, extract::State};
    use sqlx::Executor;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_PHONE_SEQUENCE: AtomicUsize = AtomicUsize::new(10_000_000);

    async fn test_db() -> Db {
        let path = std::env::temp_dir()
            .join(format!(
                "stitch-barbershop-test-{}.db",
                uuid::Uuid::new_v4()
            ))
            .display()
            .to_string()
            .replace('\\', "/");
        let db = Db::connect(&format!("sqlite://{path}?mode=rwc"))
            .await
            .unwrap();
        migrate(&db).await.unwrap();
        db
    }

    async fn admin_headers(db: &Db) -> HeaderMap {
        let Json(auth) = super::register_barbershop(
            State(db.clone()),
            Json(RegisterBarbershop {
                barbershop_name: format!("Barbearia {}", uuid::Uuid::new_v4()),
                owner_name: "Administrador".to_string(),
                email: format!("admin-{}@teste.local", uuid::Uuid::new_v4()),
                password: "TestPassword@123".to_string(),
            }),
        )
        .await
        .unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderValue::from_str(&format!("Bearer {}", auth.token)).unwrap(),
        );
        headers
    }

    #[tokio::test]
    async fn registration_works_with_legacy_admin_role_constraint() {
        let db = Db::connect("sqlite::memory:").await.unwrap();
        db.pool
            .execute(
                "
                create table users (
                    id integer primary key autoincrement,
                    name text not null,
                    email text not null unique,
                    password_hash text not null,
                    role text not null check (role in ('admin', 'barber', 'reception'))
                );
                ",
            )
            .await
            .unwrap();
        migrate(&db).await.unwrap();

        let Json(auth) = super::register_barbershop(
            State(db),
            Json(RegisterBarbershop {
                barbershop_name: "Barbearia Legacy".to_string(),
                owner_name: "Administrador".to_string(),
                email: "legacy-admin@teste.local".to_string(),
                password: "TestPassword@123".to_string(),
            }),
        )
        .await
        .unwrap();

        assert_eq!(auth.user.role, "admin");
    }

    #[tokio::test]
    async fn registration_rejects_duplicate_owner_email_with_bad_request() {
        let db = test_db().await;
        let email = format!("duplicado-{}@teste.local", uuid::Uuid::new_v4());
        let _ = super::register_barbershop(
            State(db.clone()),
            Json(RegisterBarbershop {
                barbershop_name: "Barbearia Original".to_string(),
                owner_name: "Administrador".to_string(),
                email: email.clone(),
                password: "TestPassword@123".to_string(),
            }),
        )
        .await
        .unwrap();

        let result = super::register_barbershop(
            State(db.clone()),
            Json(RegisterBarbershop {
                barbershop_name: "Barbearia Duplicada".to_string(),
                owner_name: "Outro Administrador".to_string(),
                email,
                password: "TestPassword@123".to_string(),
            }),
        )
        .await;

        assert!(matches!(result, Err(ApiError::BadRequest(message)) if message.contains("email")));
        let barbershops: (i64,) = sqlx::query_as(
            "select count(*) from barbershops where slug like 'barbearia-duplicada%'",
        )
        .fetch_one(&db.pool)
        .await
        .unwrap();
        assert_eq!(barbershops.0, 0);
    }

    #[tokio::test]
    async fn password_reset_changes_admin_password_and_consumes_token() {
        let db = test_db().await;
        let email = format!("reset-admin-{}@teste.local", uuid::Uuid::new_v4());
        let _ = super::register_barbershop(
            State(db.clone()),
            Json(RegisterBarbershop {
                barbershop_name: "Barbearia Reset".to_string(),
                owner_name: "Administrador Reset".to_string(),
                email: email.clone(),
                password: "TestPassword@123".to_string(),
            }),
        )
        .await
        .unwrap();

        let Json(requested) = super::request_password_reset(
            State(db.clone()),
            Json(PasswordResetRequest {
                email: email.clone(),
                account_type: "establishment".to_string(),
            }),
        )
        .await
        .unwrap();
        let token = requested
            .reset_token
            .expect("tests expose the generated reset token");

        let _ = super::reset_password(
            State(db.clone()),
            Json(PasswordResetConfirm {
                token: token.clone(),
                password: "NovaSenha@123".to_string(),
            }),
        )
        .await
        .unwrap();

        let old_login = super::login(
            State(db.clone()),
            Json(LoginRequest {
                email: email.clone(),
                password: "TestPassword@123".to_string(),
                account_type: "establishment".to_string(),
            }),
        )
        .await;
        assert!(matches!(old_login, Err(ApiError::Unauthorized)));

        let new_login = super::login(
            State(db.clone()),
            Json(LoginRequest {
                email,
                password: "NovaSenha@123".to_string(),
                account_type: "establishment".to_string(),
            }),
        )
        .await;
        assert!(new_login.is_ok());

        let reused = super::reset_password(
            State(db),
            Json(PasswordResetConfirm {
                token,
                password: "OutraSenha@123".to_string(),
            }),
        )
        .await;
        assert!(matches!(reused, Err(ApiError::BadRequest(message)) if message.contains("codigo")));
    }

    #[tokio::test]
    async fn password_reset_request_does_not_reveal_unknown_email() {
        let db = test_db().await;

        let Json(response) = super::request_password_reset(
            State(db.clone()),
            Json(PasswordResetRequest {
                email: "nao-existe@teste.local".to_string(),
                account_type: "establishment".to_string(),
            }),
        )
        .await
        .unwrap();

        assert!(response.reset_token.is_none());
        let tokens: (i64,) = sqlx::query_as("select count(*) from password_reset_tokens")
            .fetch_one(&db.pool)
            .await
            .unwrap();
        assert_eq!(tokens.0, 0);
    }

    #[tokio::test]
    async fn login_respects_selected_account_type() {
        let db = test_db().await;
        let admin_email = format!("admin-login-{}@teste.local", uuid::Uuid::new_v4());
        let barber_email = format!("barber-login-{}@teste.local", uuid::Uuid::new_v4());
        let Json(_) = super::register_barbershop(
            State(db.clone()),
            Json(RegisterBarbershop {
                barbershop_name: "Barbearia Tipo Login".to_string(),
                owner_name: "Administrador".to_string(),
                email: admin_email.clone(),
                password: "TestPassword@123".to_string(),
            }),
        )
        .await
        .unwrap();
        let headers = admin_headers(&db).await;
        create_barber_for(&db, headers, &barber_email).await;

        let establishment_login = super::login(
            State(db.clone()),
            Json(LoginRequest {
                email: admin_email.clone(),
                password: "TestPassword@123".to_string(),
                account_type: "establishment".to_string(),
            }),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(establishment_login.user.role, "admin");

        let professional_login = super::login(
            State(db.clone()),
            Json(LoginRequest {
                email: barber_email.clone(),
                password: "TestPassword@123".to_string(),
                account_type: "professional".to_string(),
            }),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(professional_login.user.role, "barber");

        let wrong_admin_type = super::login(
            State(db.clone()),
            Json(LoginRequest {
                email: admin_email,
                password: "TestPassword@123".to_string(),
                account_type: "professional".to_string(),
            }),
        )
        .await;
        assert!(matches!(wrong_admin_type, Err(ApiError::Unauthorized)));

        let wrong_barber_type = super::login(
            State(db),
            Json(LoginRequest {
                email: barber_email,
                password: "TestPassword@123".to_string(),
                account_type: "establishment".to_string(),
            }),
        )
        .await;
        assert!(matches!(wrong_barber_type, Err(ApiError::Unauthorized)));
    }

    #[tokio::test]
    async fn password_reset_respects_selected_account_type() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let barber_email = format!("reset-barber-{}@teste.local", uuid::Uuid::new_v4());
        let barber = create_barber_for(&db, headers, &barber_email).await;

        let Json(wrong_type) = super::request_password_reset(
            State(db.clone()),
            Json(PasswordResetRequest {
                email: barber_email.clone(),
                account_type: "establishment".to_string(),
            }),
        )
        .await
        .unwrap();
        assert!(wrong_type.reset_token.is_none());

        let Json(right_type) = super::request_password_reset(
            State(db.clone()),
            Json(PasswordResetRequest {
                email: barber_email,
                account_type: "professional".to_string(),
            }),
        )
        .await
        .unwrap();
        assert!(right_type.reset_token.is_some());

        let stored: (String, i64) = sqlx::query_as(
            "select subject_type, subject_id from password_reset_tokens where token = ?",
        )
        .bind(right_type.reset_token.unwrap())
        .fetch_one(&db.pool)
        .await
        .unwrap();
        assert_eq!(stored.0, "barber");
        assert_eq!(stored.1, barber.id);
    }

    async fn create_client_for(db: &Db, headers: HeaderMap, name: &str) -> Client {
        let phone = next_test_phone();
        let Json(client) = super::create_client(
            State(db.clone()),
            headers,
            Json(CreateClient {
                name: name.to_string(),
                phone,
                email: None,
                document: None,
                haircut_frequency: Some("Mensal".to_string()),
            }),
        )
        .await
        .unwrap();
        client
    }

    fn next_test_phone() -> String {
        let number = TEST_PHONE_SEQUENCE.fetch_add(1, Ordering::SeqCst);
        format!("119{number:08}")
    }

    async fn create_client_with_contact(
        db: &Db,
        headers: HeaderMap,
        name: &str,
        phone: &str,
        document: Option<&str>,
    ) -> ApiResult<Json<Client>> {
        super::create_client(
            State(db.clone()),
            headers,
            Json(CreateClient {
                name: name.to_string(),
                phone: phone.to_string(),
                email: None,
                document: document.map(str::to_string),
                haircut_frequency: Some("Mensal".to_string()),
            }),
        )
        .await
    }

    async fn create_service_for(db: &Db, headers: HeaderMap, name: &str) -> Service {
        let Json(service) = super::create_service(
            State(db.clone()),
            headers,
            Json(CreateService {
                name: name.to_string(),
                description: None,
                duration_minutes: 45,
                price_cents: 8_500,
                category: None,
            }),
        )
        .await
        .unwrap();
        service
    }

    #[tokio::test]
    async fn service_updating_works() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let service = create_service_for(&db, headers.clone(), "Corte Simples").await;

        let Json(updated) = super::update_service(
            State(db),
            headers,
            Path(service.id),
            Json(UpdateService {
                name: "Corte Premium".to_string(),
                description: Some("Corte e finalizacao".to_string()),
                duration_minutes: 60,
                price_cents: 12_000,
                category: Some("cabelo".to_string()),
                active: Some(true),
            }),
        )
        .await
        .unwrap();

        assert_eq!(updated.name, "Corte Premium");
        assert_eq!(updated.description, "Corte e finalizacao");
        assert_eq!(updated.duration_minutes, 60);
        assert_eq!(updated.price_cents, 12_000);
        assert_eq!(updated.category, "cabelo");
        assert!(updated.active);
    }

    #[tokio::test]
    async fn inactive_service_remains_listed_for_reactivation() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let service = create_service_for(&db, headers.clone(), "Servico Pausavel").await;

        let Json(updated) = super::update_service(
            State(db.clone()),
            headers.clone(),
            Path(service.id),
            Json(UpdateService {
                name: "Servico Pausavel".to_string(),
                description: None,
                duration_minutes: 45,
                price_cents: 8_500,
                category: None,
                active: Some(false),
            }),
        )
        .await
        .unwrap();
        assert!(!updated.active);

        let Json(services) = super::list_services(State(db), headers).await.unwrap();

        let listed = services
            .iter()
            .find(|item| item.id == service.id)
            .expect("inactive service should stay visible in admin catalog");
        assert!(!listed.active);
    }

    #[tokio::test]
    async fn tenant_a_cannot_update_tenant_b_service() {
        let db = test_db().await;
        let headers_a = admin_headers(&db).await;
        let headers_b = admin_headers(&db).await;
        let service_b = create_service_for(&db, headers_b, "Servico B").await;

        let result = super::update_service(
            State(db),
            headers_a,
            Path(service_b.id),
            Json(UpdateService {
                name: "Tentativa".to_string(),
                description: None,
                duration_minutes: 30,
                price_cents: 5_000,
                category: None,
                active: Some(true),
            }),
        )
        .await;

        assert!(matches!(result, Err(ApiError::NotFound)));
    }

    async fn create_barber_for(db: &Db, headers: HeaderMap, email: &str) -> Barber {
        create_barber_with_document(db, headers, email, &next_test_document()).await
    }

    async fn create_barber_with_document(
        db: &Db,
        headers: HeaderMap,
        email: &str,
        document: &str,
    ) -> Barber {
        let Json(barber) = super::create_barber(
            State(db.clone()),
            headers,
            Json(CreateBarber {
                name: format!("Profissional {email}"),
                email: email.to_string(),
                document: document.to_string(),
                password: "TestPassword@123".to_string(),
                specialty: None,
            }),
        )
        .await
        .unwrap();
        barber
    }

    fn next_test_document() -> String {
        let base = TEST_PHONE_SEQUENCE.fetch_add(1, Ordering::SeqCst) as u64;
        test_cpf_from_base(100_000_000 + (base % 800_000_000))
    }

    fn test_cpf_from_base(base: u64) -> String {
        let base = format!("{:09}", base % 1_000_000_000);
        let first = cpf_test_digit(&base, 9);
        let ten_digits = format!("{base}{first}");
        let second = cpf_test_digit(&ten_digits, 10);
        format!("{ten_digits}{second}")
    }

    fn cpf_test_digit(digits: &str, len: usize) -> u8 {
        let sum: u32 = digits
            .as_bytes()
            .iter()
            .take(len)
            .enumerate()
            .map(|(index, digit)| u32::from(digit - b'0') * ((len + 1 - index) as u32))
            .sum();
        let remainder = sum % 11;
        if remainder < 2 {
            0
        } else {
            (11 - remainder) as u8
        }
    }

    #[tokio::test]
    async fn client_updating_works() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let Json(client) = create_client_with_contact(
            &db,
            headers.clone(),
            "Cliente Antigo",
            "(11) 99999-1111",
            Some("111.444.777-35"),
        )
        .await
        .unwrap();

        let Json(updated) = super::update_client(
            State(db),
            headers,
            Path(client.id),
            Json(UpdateClient {
                name: "Cliente Novo".to_string(),
                phone: "(11) 99999-2222".to_string(),
                email: Some("novo@teste.com".to_string()),
                document: Some("529.982.247-25".to_string()),
                haircut_frequency: Some("Quinzenal".to_string()),
            }),
        )
        .await
        .unwrap();

        assert_eq!(updated.name, "Cliente Novo");
        assert_eq!(updated.phone, "11999992222");
        assert_eq!(updated.email.unwrap(), "novo@teste.com");
        assert_eq!(updated.document.unwrap(), "52998224725");
        assert_eq!(updated.haircut_frequency.unwrap(), "Quinzenal");
    }

    #[tokio::test]
    async fn client_updating_avoids_self_phone_conflict() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let Json(client) = create_client_with_contact(
            &db,
            headers.clone(),
            "Cliente Fiel",
            "(11) 99999-3333",
            Some("935.411.347-80"),
        )
        .await
        .unwrap();

        let result = super::update_client(
            State(db),
            headers,
            Path(client.id),
            Json(UpdateClient {
                name: "Cliente Fiel Nome Alterado".to_string(),
                phone: "(11) 99999-3333".to_string(),
                email: None,
                document: Some("93541134780".to_string()),
                haircut_frequency: Some("Mensal".to_string()),
            }),
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn client_updating_rejects_duplicate_phone() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let _ =
            create_client_with_contact(&db, headers.clone(), "Cliente A", "(11) 99999-4444", None)
                .await
                .unwrap();
        let Json(client_b) =
            create_client_with_contact(&db, headers.clone(), "Cliente B", "(11) 99999-5555", None)
                .await
                .unwrap();

        let result = super::update_client(
            State(db),
            headers,
            Path(client_b.id),
            Json(UpdateClient {
                name: "Cliente B tentando roubar fone".to_string(),
                phone: "(11) 99999-4444".to_string(),
                email: None,
                document: None,
                haircut_frequency: Some("Mensal".to_string()),
            }),
        )
        .await;

        assert!(
            matches!(result, Err(ApiError::BadRequest(message)) if message.contains("telefone"))
        );
    }

    #[tokio::test]
    async fn client_phone_must_be_unique_in_barbershop() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let _ = create_client_with_contact(
            &db,
            headers.clone(),
            "Cliente Original",
            "(11) 99999-8888",
            None,
        )
        .await
        .unwrap();

        let result =
            create_client_with_contact(&db, headers, "Outro Nome", "11999998888", None).await;

        assert!(
            matches!(result, Err(ApiError::BadRequest(message)) if message.contains("telefone"))
        );
    }

    #[tokio::test]
    async fn client_document_must_be_unique_when_present() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let _ = create_client_with_contact(
            &db,
            headers.clone(),
            "Cliente Original",
            "(11) 99999-8888",
            Some("123.456.789-09"),
        )
        .await
        .unwrap();

        let result = create_client_with_contact(
            &db,
            headers,
            "Outro Cliente",
            "(11) 99999-7777",
            Some("12345678909"),
        )
        .await;

        assert!(matches!(result, Err(ApiError::BadRequest(message)) if message.contains("CPF")));
    }

    #[tokio::test]
    async fn client_document_must_be_valid_when_present() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;

        let result = create_client_with_contact(
            &db,
            headers,
            "Cliente CPF Invalido",
            "(11) 99999-6666",
            Some("123.456.789-00"),
        )
        .await;

        assert!(matches!(result, Err(ApiError::BadRequest(message)) if message.contains("CPF")));
    }

    #[tokio::test]
    async fn client_updating_rejects_invalid_document() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let Json(client) = create_client_with_contact(
            &db,
            headers.clone(),
            "Cliente Com CPF",
            "(11) 99999-6666",
            Some("111.444.777-35"),
        )
        .await
        .unwrap();

        let result = super::update_client(
            State(db),
            headers,
            Path(client.id),
            Json(UpdateClient {
                name: "Cliente Com CPF".to_string(),
                phone: "(11) 99999-6666".to_string(),
                email: None,
                document: Some("111.111.111-11".to_string()),
                haircut_frequency: Some("Mensal".to_string()),
            }),
        )
        .await;

        assert!(matches!(result, Err(ApiError::BadRequest(message)) if message.contains("CPF")));
    }

    #[tokio::test]
    async fn blank_client_document_does_not_block_multiple_clients() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let _ = create_client_with_contact(
            &db,
            headers.clone(),
            "Cliente Original",
            "(11) 99999-8888",
            None,
        )
        .await
        .unwrap();

        let result = create_client_with_contact(
            &db,
            headers,
            "Outro Cliente",
            "(11) 99999-7777",
            Some("   "),
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn barber_email_must_be_unique() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        create_barber_for(&db, headers.clone(), "profissional@teste.local").await;

        let result = super::create_barber(
            State(db),
            headers,
            Json(CreateBarber {
                name: "Outro Profissional".to_string(),
                email: "PROFISSIONAL@teste.local".to_string(),
                document: "111.444.777-35".to_string(),
                password: "TestPassword@123".to_string(),
                specialty: None,
            }),
        )
        .await;

        assert!(matches!(result, Err(ApiError::BadRequest(message)) if message.contains("email")));
    }

    #[tokio::test]
    async fn barber_document_must_be_valid() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;

        let result = super::create_barber(
            State(db),
            headers,
            Json(CreateBarber {
                name: "Profissional CPF Invalido".to_string(),
                email: "cpf-invalido@teste.local".to_string(),
                document: "123.456.789-00".to_string(),
                password: "TestPassword@123".to_string(),
                specialty: None,
            }),
        )
        .await;

        assert!(matches!(result, Err(ApiError::BadRequest(message)) if message.contains("CPF")));
    }

    #[tokio::test]
    async fn barber_document_must_be_unique_and_linked_to_email() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        create_barber_with_document(
            &db,
            headers.clone(),
            "cpf-base@teste.local",
            "123.456.789-09",
        )
        .await;

        let result = super::create_barber(
            State(db),
            headers,
            Json(CreateBarber {
                name: "Outro CPF".to_string(),
                email: "outro-cpf@teste.local".to_string(),
                document: "123.456.789-09".to_string(),
                password: "TestPassword@123".to_string(),
                specialty: None,
            }),
        )
        .await;

        assert!(
            matches!(result, Err(ApiError::BadRequest(message)) if message.contains("CPF") && message.contains("email"))
        );
    }

    async fn barber_headers(db: &Db, email: &str) -> HeaderMap {
        let Json(auth) = super::login(
            State(db.clone()),
            Json(LoginRequest {
                email: email.to_string(),
                password: "TestPassword@123".to_string(),
                account_type: "professional".to_string(),
            }),
        )
        .await
        .unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderValue::from_str(&format!("Bearer {}", auth.token)).unwrap(),
        );
        headers
    }

    #[test]
    fn checkout_change_is_exact() {
        let subtotal = 16_000;
        let total = subtotal + 2_000;
        assert_eq!(20_000 - total, 2_000);
    }

    #[test]
    fn client_phone_must_be_mobile_with_area_code() {
        assert_eq!(
            super::normalize_mobile_phone("(11) 99999-8888"),
            Some("11999998888".to_string())
        );
        assert_eq!(
            super::normalize_mobile_phone("11999998888"),
            Some("11999998888".to_string())
        );
        assert_eq!(super::normalize_mobile_phone("1133334444"), None);
        assert_eq!(super::normalize_mobile_phone("11888887777"), None);
        assert_eq!(super::normalize_mobile_phone("999998888"), None);
    }

    #[test]
    fn client_phone_accepts_repeated_digit_placeholders() {
        assert_eq!(
            super::normalize_mobile_phone("99999999"),
            Some("99999999".to_string())
        );
        assert_eq!(
            super::normalize_mobile_phone("888"),
            Some("888".to_string())
        );
        assert_eq!(
            super::normalize_mobile_phone("777"),
            Some("777".to_string())
        );
        for digit in '0'..='9' {
            let placeholder = digit.to_string().repeat(12);
            assert_eq!(
                super::normalize_mobile_phone(&placeholder),
                Some(placeholder)
            );
        }
    }

    #[test]
    fn haircut_frequency_is_required_display_text() {
        assert_eq!(
            super::required_display_text(Some("A cada 15 dias"), "frequencia").unwrap(),
            "A cada 15 dias"
        );
        assert!(super::required_display_text(Some("   "), "frequencia").is_err());
        assert!(super::required_display_text(None, "frequencia").is_err());
    }

    #[test]
    fn optional_password_update_ignores_blank_values() {
        assert_eq!(super::optional_password_update(None), None);
        assert_eq!(super::optional_password_update(Some("   ")), None);
        assert_eq!(
            super::optional_password_update(Some("123456")),
            Some("123456")
        );
    }

    #[test]
    fn financial_profit_discounts_commissions_and_expenses() {
        assert_eq!(super::financial_profit(100_000, 30_000, 12_500), 57_500);
    }

    #[test]
    fn display_text_is_trimmed_and_rejects_html_or_control_chars() {
        assert_eq!(
            super::required_display_text(Some("  Ana Paula  "), "nome").unwrap(),
            "Ana Paula"
        );
        assert!(super::required_display_text(Some("<script>alert(1)</script>"), "nome").is_err());
        assert!(super::required_display_text(Some("Ana\nPaula"), "nome").is_err());
        assert!(super::required_display_text(Some("  "), "nome").is_err());
    }

    #[test]
    fn optional_text_is_trimmed_empty_or_rejected_when_unsafe() {
        assert_eq!(
            super::optional_display_text(Some("  Barba premium  "), "descricao").unwrap(),
            Some("Barba premium".to_string())
        );
        assert_eq!(
            super::optional_display_text(Some("  "), "descricao").unwrap(),
            None
        );
        assert_eq!(
            super::optional_display_text(None, "descricao").unwrap(),
            None
        );
        assert!(
            super::optional_display_text(Some("<img src=x onerror=alert(1)>"), "descricao")
                .is_err()
        );
    }

    #[test]
    fn monetary_values_are_validated_on_the_server() {
        assert!(super::validate_non_negative_cents(0, "desconto").is_ok());
        assert!(super::validate_non_negative_cents(500, "gorjeta").is_ok());
        assert!(super::validate_non_negative_cents(-1, "valor pago").is_err());
        assert!(super::validate_positive_cents(1, "gasto").is_ok());
        assert!(super::validate_positive_cents(0, "gasto").is_err());
    }

    #[test]
    fn payment_method_is_strictly_whitelisted() {
        assert_eq!(super::validate_payment_method(" pix ").unwrap(), "pix");
        assert_eq!(super::validate_payment_method("credit").unwrap(), "credit");
        assert!(super::validate_payment_method("pix'; drop table payments; --").is_err());
        assert!(super::validate_payment_method("voucher").is_err());
    }

    #[test]
    fn appointment_start_must_be_valid_business_hour() {
        assert_eq!(
            super::validate_appointment_start("2026-05-28T09:00").unwrap(),
            "2026-05-28T09:00"
        );
        assert_eq!(
            super::validate_appointment_start("2026-05-28T21:30").unwrap(),
            "2026-05-28T21:30"
        );
        assert!(super::validate_appointment_start("2026-05-28T08:59").is_err());
        assert!(super::validate_appointment_start("2026-05-28T22:00").is_err());
        assert!(super::validate_appointment_start("not a date").is_err());
    }

    #[test]
    fn ids_must_be_positive() {
        assert!(super::validate_positive_id(1, "cliente").is_ok());
        assert!(super::validate_positive_id(0, "cliente").is_err());
        assert!(super::validate_positive_id(-4, "servico").is_err());
    }

    #[test]
    fn service_ids_must_be_present_positive_and_unique() {
        assert!(super::validate_service_ids(&[1, 2, 3]).is_ok());
        assert!(super::validate_service_ids(&[]).is_err());
        assert!(super::validate_service_ids(&[1, 1]).is_err());
        assert!(super::validate_service_ids(&[1, 0]).is_err());
    }

    #[tokio::test]
    async fn client_sql_injection_payload_is_data_not_sql() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let payload = "Robert'); DROP TABLE clients;--";
        let Json(client) = super::create_client(
            State(db.clone()),
            headers,
            Json(CreateClient {
                name: payload.to_string(),
                phone: "(11) 99999-8888".to_string(),
                email: None,
                document: None,
                haircut_frequency: Some("Mensal".to_string()),
            }),
        )
        .await
        .unwrap();

        assert_eq!(client.name, payload);
        let clients_table_still_exists: (i64,) = sqlx::query_as("select count(*) from clients")
            .fetch_one(&db.pool)
            .await
            .unwrap();
        assert_eq!(clients_table_still_exists.0, 1);
    }

    #[tokio::test]
    async fn client_creation_rejects_xss_name_on_the_server() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let result = super::create_client(
            State(db),
            headers,
            Json(CreateClient {
                name: "<script>alert(1)</script>".to_string(),
                phone: "(11) 99999-8888".to_string(),
                email: None,
                document: None,
                haircut_frequency: Some("Mensal".to_string()),
            }),
        )
        .await;

        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[tokio::test]
    async fn appointment_creation_rejects_missing_relations_on_the_server() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let result = super::create_appointment(
            State(db),
            headers,
            Json(CreateAppointment {
                client_id: 999,
                barber_id: 999,
                service_ids: vec![999],
                starts_at: "2026-05-28T09:00".to_string(),
            }),
        )
        .await;

        assert!(matches!(result, Err(ApiError::NotFound)));
    }

    #[tokio::test]
    async fn appointment_updating_recalculates_services_and_status() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let client = create_client_for(&db, headers.clone(), "Cliente Agenda").await;
        let barber = create_barber_for(&db, headers.clone(), "agenda@teste.local").await;
        let service_a = create_service_for(&db, headers.clone(), "Corte Agenda").await;
        let service_b = create_service_for(&db, headers.clone(), "Barba Agenda").await;
        let Json(appointment) = super::create_appointment(
            State(db.clone()),
            headers.clone(),
            Json(CreateAppointment {
                client_id: client.id,
                barber_id: barber.id,
                service_ids: vec![service_a.id],
                starts_at: "2026-05-28T09:00".to_string(),
            }),
        )
        .await
        .unwrap();

        let Json(updated) = super::update_appointment(
            State(db.clone()),
            headers,
            Path(appointment.id),
            Json(UpdateAppointment {
                client_id: client.id,
                barber_id: barber.id,
                service_ids: vec![service_a.id, service_b.id],
                starts_at: "2026-05-28T10:00".to_string(),
                status: "in_chair".to_string(),
            }),
        )
        .await
        .unwrap();

        assert_eq!(updated.starts_at, "2026-05-28T10:00");
        assert_eq!(updated.status, "in_chair");
        assert_eq!(
            updated.total_cents,
            service_a.price_cents + service_b.price_cents
        );
        assert!(updated.services.contains("Corte Agenda"));
        assert!(updated.services.contains("Barba Agenda"));

        let service_count: (i64,) =
            sqlx::query_as("select count(*) from appointment_services where appointment_id = ?")
                .bind(appointment.id)
                .fetch_one(&db.pool)
                .await
                .unwrap();
        assert_eq!(service_count.0, 2);
    }

    #[tokio::test]
    async fn completed_appointment_cannot_be_updated() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let client = create_client_for(&db, headers.clone(), "Cliente Completo").await;
        let barber = create_barber_for(&db, headers.clone(), "complete-update@teste.local").await;
        let service = create_service_for(&db, headers.clone(), "Corte Completo").await;
        let Json(appointment) = super::create_appointment(
            State(db.clone()),
            headers.clone(),
            Json(CreateAppointment {
                client_id: client.id,
                barber_id: barber.id,
                service_ids: vec![service.id],
                starts_at: "2026-05-28T09:00".to_string(),
            }),
        )
        .await
        .unwrap();
        let _ = super::checkout(
            State(db.clone()),
            headers.clone(),
            Json(CheckoutRequest {
                appointment_id: appointment.id,
                payment_method: "pix".to_string(),
                paid_cents: appointment.total_cents,
                tip_cents: 0,
                discount_cents: 0,
                payments: vec![],
            }),
        )
        .await
        .unwrap();

        let result = super::update_appointment(
            State(db),
            headers,
            Path(appointment.id),
            Json(UpdateAppointment {
                client_id: client.id,
                barber_id: barber.id,
                service_ids: vec![service.id],
                starts_at: "2026-05-28T10:00".to_string(),
                status: "cancelled".to_string(),
            }),
        )
        .await;

        assert!(
            matches!(result, Err(ApiError::BadRequest(message)) if message.contains("finalizado"))
        );
    }

    #[tokio::test]
    async fn checkout_rejects_negative_values_before_writing_payment() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let result = super::checkout(
            State(db.clone()),
            headers,
            Json(CheckoutRequest {
                appointment_id: 1,
                payment_method: "pix".to_string(),
                paid_cents: 10_000,
                tip_cents: -1,
                discount_cents: 0,
                payments: vec![],
            }),
        )
        .await;

        assert!(matches!(result, Err(ApiError::BadRequest(_))));
        let payments: (i64,) = sqlx::query_as("select count(*) from payments")
            .fetch_one(&db.pool)
            .await
            .unwrap();
        assert_eq!(payments.0, 0);
    }

    #[tokio::test]
    async fn checkout_rejects_completed_appointment_without_duplicate_payment() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let client = create_client_for(&db, headers.clone(), "Cliente Checkout").await;
        let barber = create_barber_for(&db, headers.clone(), "checkout@teste.local").await;
        let service = create_service_for(&db, headers.clone(), "Corte Checkout").await;
        let Json(appointment) = super::create_appointment(
            State(db.clone()),
            headers.clone(),
            Json(CreateAppointment {
                client_id: client.id,
                barber_id: barber.id,
                service_ids: vec![service.id],
                starts_at: "2026-05-28T10:00".to_string(),
            }),
        )
        .await
        .unwrap();

        let Json(first_checkout) = super::checkout(
            State(db.clone()),
            headers.clone(),
            Json(CheckoutRequest {
                appointment_id: appointment.id,
                payment_method: "pix".to_string(),
                paid_cents: appointment.total_cents,
                tip_cents: 0,
                discount_cents: 0,
                payments: vec![],
            }),
        )
        .await
        .unwrap();
        assert_eq!(first_checkout.total_cents, appointment.total_cents);

        let second_checkout = super::checkout(
            State(db.clone()),
            headers,
            Json(CheckoutRequest {
                appointment_id: appointment.id,
                payment_method: "pix".to_string(),
                paid_cents: appointment.total_cents,
                tip_cents: 0,
                discount_cents: 0,
                payments: vec![],
            }),
        )
        .await;

        assert!(matches!(second_checkout, Err(ApiError::BadRequest(_))));
        let payments: (i64,) =
            sqlx::query_as("select count(*) from payments where appointment_id = ?")
                .bind(appointment.id)
                .fetch_one(&db.pool)
                .await
                .unwrap();
        assert_eq!(payments.0, 1);
    }

    #[tokio::test]
    async fn checkout_split_payments_turn_overpaid_amount_into_barber_tip() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let client = create_client_for(&db, headers.clone(), "Cliente Split").await;
        let barber = create_barber_for(&db, headers.clone(), "split@teste.local").await;
        let service = create_service_for(&db, headers.clone(), "Corte Split").await;
        let Json(appointment) = super::create_appointment(
            State(db.clone()),
            headers.clone(),
            Json(CreateAppointment {
                client_id: client.id,
                barber_id: barber.id,
                service_ids: vec![service.id],
                starts_at: "2026-05-28T11:00".to_string(),
            }),
        )
        .await
        .unwrap();

        let Json(checkout) = super::checkout(
            State(db.clone()),
            headers,
            Json(CheckoutRequest {
                appointment_id: appointment.id,
                payment_method: "cash".to_string(),
                paid_cents: 10_000,
                tip_cents: 0,
                discount_cents: 0,
                payments: vec![
                    CheckoutPaymentInput {
                        method: "cash".to_string(),
                        amount_cents: 5_000,
                    },
                    CheckoutPaymentInput {
                        method: "pix".to_string(),
                        amount_cents: 5_000,
                    },
                ],
            }),
        )
        .await
        .unwrap();

        assert_eq!(checkout.subtotal_cents, 8_500);
        assert_eq!(checkout.paid_cents, 10_000);
        assert_eq!(checkout.tip_cents, 1_500);
        assert_eq!(checkout.change_cents, 0);
        assert_eq!(checkout.total_cents, 10_000);

        let splits: Vec<(String, i64)> = sqlx::query_as(
            "select ps.method, ps.amount_cents
             from payment_splits ps
             join payments p on p.id = ps.payment_id
             where p.appointment_id = ?
             order by ps.id",
        )
        .bind(appointment.id)
        .fetch_all(&db.pool)
        .await
        .unwrap();
        assert_eq!(
            splits,
            vec![("cash".to_string(), 5_000), ("pix".to_string(), 5_000)]
        );

        let barber_tips: (i64,) =
            sqlx::query_as("select monthly_tips_cents from barbers where id = ?")
                .bind(barber.id)
                .fetch_one(&db.pool)
                .await
                .unwrap();
        assert_eq!(barber_tips.0, 1_500);
    }

    #[tokio::test]
    async fn checkout_split_payments_reject_underpaid_total_without_writing_payment() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let client = create_client_for(&db, headers.clone(), "Cliente Underpaid").await;
        let barber = create_barber_for(&db, headers.clone(), "underpaid@teste.local").await;
        let service = create_service_for(&db, headers.clone(), "Corte Underpaid").await;
        let Json(appointment) = super::create_appointment(
            State(db.clone()),
            headers.clone(),
            Json(CreateAppointment {
                client_id: client.id,
                barber_id: barber.id,
                service_ids: vec![service.id],
                starts_at: "2026-05-28T12:00".to_string(),
            }),
        )
        .await
        .unwrap();

        let result = super::checkout(
            State(db.clone()),
            headers,
            Json(CheckoutRequest {
                appointment_id: appointment.id,
                payment_method: "pix".to_string(),
                paid_cents: 8_000,
                tip_cents: 0,
                discount_cents: 0,
                payments: vec![CheckoutPaymentInput {
                    method: "pix".to_string(),
                    amount_cents: 8_000,
                }],
            }),
        )
        .await;

        assert!(
            matches!(result, Err(ApiError::BadRequest(message)) if message.contains("insuficiente"))
        );
        let payments: (i64,) =
            sqlx::query_as("select count(*) from payments where appointment_id = ?")
                .bind(appointment.id)
                .fetch_one(&db.pool)
                .await
                .unwrap();
        assert_eq!(payments.0, 0);
    }

    #[tokio::test]
    async fn tenant_a_cannot_list_tenant_b_clients() {
        let db = test_db().await;
        let headers_a = admin_headers(&db).await;
        let headers_b = admin_headers(&db).await;
        create_client_for(&db, headers_a.clone(), "Cliente A").await;
        create_client_for(&db, headers_b.clone(), "Cliente B").await;

        let Json(clients_a) = super::list_clients(State(db.clone()), headers_a)
            .await
            .unwrap();
        let Json(clients_b) = super::list_clients(State(db), headers_b).await.unwrap();

        assert_eq!(clients_a.len(), 1);
        assert_eq!(clients_a[0].name, "Cliente A");
        assert_eq!(clients_b.len(), 1);
        assert_eq!(clients_b[0].name, "Cliente B");
    }

    #[tokio::test]
    async fn tenant_a_cannot_create_appointment_with_tenant_b_records() {
        let db = test_db().await;
        let headers_a = admin_headers(&db).await;
        let headers_b = admin_headers(&db).await;
        let client_b = create_client_for(&db, headers_b.clone(), "Cliente B").await;
        let service_b = create_service_for(&db, headers_b.clone(), "Servico B").await;
        let barber_b = create_barber_for(&db, headers_b, "barber-b@teste.local").await;

        let result = super::create_appointment(
            State(db),
            headers_a,
            Json(CreateAppointment {
                client_id: client_b.id,
                barber_id: barber_b.id,
                service_ids: vec![service_b.id],
                starts_at: "2026-05-28T09:00".to_string(),
            }),
        )
        .await;

        assert!(matches!(result, Err(ApiError::NotFound)));
    }

    #[tokio::test]
    async fn barber_lists_only_own_appointments() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let service = create_service_for(&db, headers.clone(), "Corte").await;
        let barber_one = create_barber_for(&db, headers.clone(), "one@teste.local").await;
        let barber_two = create_barber_for(&db, headers.clone(), "two@teste.local").await;
        let client_one = create_client_for(&db, headers.clone(), "Cliente Um").await;
        let client_two = create_client_for(&db, headers.clone(), "Cliente Dois").await;
        let _ = super::create_appointment(
            State(db.clone()),
            headers.clone(),
            Json(CreateAppointment {
                client_id: client_one.id,
                barber_id: barber_one.id,
                service_ids: vec![service.id],
                starts_at: "2026-05-28T09:00".to_string(),
            }),
        )
        .await
        .unwrap();
        let _ = super::create_appointment(
            State(db.clone()),
            headers,
            Json(CreateAppointment {
                client_id: client_two.id,
                barber_id: barber_two.id,
                service_ids: vec![service.id],
                starts_at: "2026-05-28T10:00".to_string(),
            }),
        )
        .await
        .unwrap();

        let barber_one_headers = barber_headers(&db, "one@teste.local").await;
        let Json(appointments) = super::list_appointments(State(db), barber_one_headers)
            .await
            .unwrap();

        assert_eq!(appointments.len(), 1);
        assert_eq!(appointments[0].barber_id, barber_one.id);
    }

    #[tokio::test]
    async fn barber_can_read_but_not_update_own_commissions() {
        let db = test_db().await;
        let headers = admin_headers(&db).await;
        let service = create_service_for(&db, headers.clone(), "Corte").await;
        let barber = create_barber_for(&db, headers, "commission@teste.local").await;
        let barber_headers = barber_headers(&db, "commission@teste.local").await;

        let Json(commissions) =
            super::list_commissions(State(db.clone()), barber_headers.clone(), Path(barber.id))
                .await
                .unwrap();
        let update = super::update_commission(
            State(db),
            barber_headers,
            Path(barber.id),
            Json(CommissionInput {
                service_id: service.id,
                commission_percent: 80,
            }),
        )
        .await;

        assert_eq!(commissions.len(), 1);
        assert!(matches!(update, Err(ApiError::Forbidden)));
    }
}
