use crate::{db::Db, error::ApiError};
use chrono::{Duration, NaiveDateTime, Utc};
use reqwest::StatusCode as ReqwestStatusCode;
use serde::Deserialize;
use sqlx::FromRow;

const DEFAULT_THRESHOLD: i64 = 5;
const DEFAULT_LOCKOUT_MINUTES: i64 = 15;
const DEFAULT_WINDOW_MINUTES: i64 = 15;
const DEFAULT_SITE_KEY: &str = "10000000-ffff-ffff-ffff-000000000001";
const DEFAULT_SECRET_KEY: &str = "0x0000000000000000000000000000000000000000";
const DEFAULT_VERIFY_URL: &str = "https://api.hcaptcha.com/siteverify";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthChallengeAction {
    Login,
    PasswordReset,
}

impl AuthChallengeAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::PasswordReset => "password-reset",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthChallengeDecision {
    pub required: bool,
    pub retry_after_seconds: Option<i64>,
}

#[derive(Debug, FromRow)]
struct AuthChallengeAttempt {
    failures: i64,
    locked_until: Option<String>,
    last_failed_at: String,
}

#[derive(Debug, Deserialize)]
struct HcaptchaVerificationResponse {
    success: bool,
    #[serde(default, rename = "error-codes")]
    error_codes: Vec<String>,
}

pub async fn current_auth_challenge(
    db: &Db,
    action: AuthChallengeAction,
    account_type: &str,
    identifier: &str,
) -> Result<AuthChallengeDecision, ApiError> {
    let scope = challenge_scope(action, account_type)?;
    current_auth_challenge_for_scope(db, &scope, identifier).await
}

pub async fn record_auth_failure(
    db: &Db,
    action: AuthChallengeAction,
    account_type: &str,
    identifier: &str,
) -> Result<AuthChallengeDecision, ApiError> {
    let scope = challenge_scope(action, account_type)?;
    record_auth_failure_for_scope(db, &scope, identifier).await
}

pub async fn clear_auth_attempts(
    db: &Db,
    action: AuthChallengeAction,
    account_type: &str,
    identifier: &str,
) -> Result<(), ApiError> {
    let scope = challenge_scope(action, account_type)?;
    sqlx::query("delete from auth_challenge_attempts where scope = ? and identifier = ?")
        .bind(scope)
        .bind(normalize_identifier(identifier)?)
        .execute(&db.pool)
        .await?;
    Ok(())
}

pub async fn verify_hcaptcha(token: &str) -> Result<(), ApiError> {
    let token = token.trim();
    if token.is_empty() {
        return Err(ApiError::BadRequest("validacao humana invalida".into()));
    }

    let client = reqwest::Client::new();
    let response = client
        .post(hcaptcha_verify_url())
        .form(&[
            ("secret", hcaptcha_secret()),
            ("response", token.to_string()),
            ("sitekey", hcaptcha_site_key()),
        ])
        .send()
        .await
        .map_err(|_| ApiError::BadRequest("nao foi possivel validar o captcha".into()))?;

    if response.status() != ReqwestStatusCode::OK {
        return Err(ApiError::BadRequest("nao foi possivel validar o captcha".into()));
    }

    let verification = response
        .json::<HcaptchaVerificationResponse>()
        .await
        .map_err(|_| ApiError::BadRequest("nao foi possivel validar o captcha".into()))?;

    if verification.success {
        return Ok(());
    }

    tracing::warn!(error_codes = ?verification.error_codes, "hcaptcha verification rejected");
    Err(ApiError::BadRequest("validacao humana invalida".into()))
}

pub fn challenge_required_error(action: AuthChallengeAction, retry_after_seconds: i64) -> ApiError {
    ApiError::challenge_required(action.as_str(), retry_after_seconds)
}

async fn current_auth_challenge_for_scope(
    db: &Db,
    scope: &str,
    identifier: &str,
) -> Result<AuthChallengeDecision, ApiError> {
    let identifier = normalize_identifier(identifier)?;
    let now = Utc::now().naive_utc();
    let attempt = load_attempt(db, scope, &identifier).await?;
    Ok(auth_challenge_decision(attempt.as_ref(), now, false))
}

async fn record_auth_failure_for_scope(
    db: &Db,
    scope: &str,
    identifier: &str,
) -> Result<AuthChallengeDecision, ApiError> {
    let identifier = normalize_identifier(identifier)?;
    let now = Utc::now().naive_utc();
    let attempt = load_attempt(db, scope, &identifier).await?;
    let decision = auth_challenge_decision(attempt.as_ref(), now, true);
    if decision.required && attempt.as_ref().is_some_and(|attempt| locked_until(attempt).is_some()) {
        return Ok(decision);
    }

    let failures = next_failure_count(attempt.as_ref(), now);
    let locked_until = if failures >= challenge_threshold() {
        Some(now + Duration::minutes(lockout_minutes()))
    } else {
        None
    };
    let locked_until_string = locked_until.map(format_timestamp);
    let last_failed_at_string = format_timestamp(now);

    sqlx::query(
        "insert into auth_challenge_attempts (scope, identifier, failures, locked_until, last_failed_at)
         values (?, ?, ?, ?, ?)
         on conflict(scope, identifier) do update set
            failures = excluded.failures,
            locked_until = excluded.locked_until,
            last_failed_at = excluded.last_failed_at",
    )
    .bind(scope)
    .bind(&identifier)
    .bind(failures)
    .bind(locked_until_string.clone())
    .bind(&last_failed_at_string)
    .execute(&db.pool)
    .await?;

    Ok(auth_challenge_decision(
        Some(&AuthChallengeAttempt {
            failures,
            locked_until: locked_until_string,
            last_failed_at: last_failed_at_string,
        }),
        now,
        false,
    ))
}

async fn load_attempt(
    db: &Db,
    scope: &str,
    identifier: &str,
) -> Result<Option<AuthChallengeAttempt>, ApiError> {
    let attempt = sqlx::query_as::<_, AuthChallengeAttempt>(
        "select failures, locked_until, last_failed_at
         from auth_challenge_attempts
         where scope = ? and identifier = ?",
    )
    .bind(scope)
    .bind(identifier)
    .fetch_optional(&db.pool)
    .await?;
    Ok(attempt)
}

fn auth_challenge_decision(
    attempt: Option<&AuthChallengeAttempt>,
    now: NaiveDateTime,
    force_lock: bool,
) -> AuthChallengeDecision {
    let Some(attempt) = attempt else {
        return AuthChallengeDecision {
            required: false,
            retry_after_seconds: None,
        };
    };

    if let Some(locked_until) = locked_until(attempt) {
        let retry_after_seconds = (locked_until - now).num_seconds().max(0);
        if force_lock || retry_after_seconds > 0 {
            return AuthChallengeDecision {
                required: true,
                retry_after_seconds: Some(retry_after_seconds),
            };
        }
    }

    AuthChallengeDecision {
        required: false,
        retry_after_seconds: None,
    }
}

fn next_failure_count(attempt: Option<&AuthChallengeAttempt>, now: NaiveDateTime) -> i64 {
    let Some(attempt) = attempt else {
        return 1;
    };

    let failures = if now - last_failed_at(attempt) > Duration::minutes(window_minutes()) {
        0
    } else {
        attempt.failures
    };
    failures + 1
}

fn challenge_scope(action: AuthChallengeAction, account_type: &str) -> Result<String, ApiError> {
    let account_type = normalize_account_type(account_type)?;
    Ok(format!("{}:{account_type}", action.as_str()))
}

fn normalize_account_type(value: &str) -> Result<&'static str, ApiError> {
    match value.trim() {
        "establishment" => Ok("establishment"),
        "professional" => Ok("professional"),
        _ => Err(ApiError::BadRequest("tipo de acesso invalido".into())),
    }
}

fn normalize_identifier(value: &str) -> Result<String, ApiError> {
    let value = value.trim().to_lowercase();
    if value.is_empty() {
        return Err(ApiError::BadRequest("email invalido".into()));
    }
    Ok(value)
}

fn locked_until(attempt: &AuthChallengeAttempt) -> Option<NaiveDateTime> {
    attempt
        .locked_until
        .as_deref()
        .and_then(|value| parse_timestamp(value).ok())
}

fn last_failed_at(attempt: &AuthChallengeAttempt) -> NaiveDateTime {
    parse_timestamp(&attempt.last_failed_at).unwrap_or_else(|_| Utc::now().naive_utc())
}

fn parse_timestamp(value: &str) -> Result<NaiveDateTime, ApiError> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
        .map_err(|_| ApiError::BadRequest("data invalida".into()))
}

fn format_timestamp(value: NaiveDateTime) -> String {
    value.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn challenge_threshold() -> i64 {
    env_i64("AUTH_CHALLENGE_THRESHOLD", DEFAULT_THRESHOLD)
}

fn lockout_minutes() -> i64 {
    env_i64("AUTH_CHALLENGE_LOCKOUT_MINUTES", DEFAULT_LOCKOUT_MINUTES)
}

fn window_minutes() -> i64 {
    env_i64("AUTH_CHALLENGE_WINDOW_MINUTES", DEFAULT_WINDOW_MINUTES)
}

fn hcaptcha_site_key() -> String {
    std::env::var("HCAPTCHA_SITE_KEY").unwrap_or_else(|_| DEFAULT_SITE_KEY.to_string())
}

fn hcaptcha_secret() -> String {
    std::env::var("HCAPTCHA_SECRET").unwrap_or_else(|_| DEFAULT_SECRET_KEY.to_string())
}

fn hcaptcha_verify_url() -> String {
    std::env::var("HCAPTCHA_VERIFY_URL").unwrap_or_else(|_| DEFAULT_VERIFY_URL.to_string())
}

fn env_i64(key: &str, default: i64) -> i64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Db, migrate};

    #[tokio::test]
    async fn auth_challenge_trips_after_repeated_failures_and_clears_on_success() {
        let db = Db::connect("sqlite::memory:").await.unwrap();
        migrate(&db).await.unwrap();

        let scope = AuthChallengeAction::Login;
        let identifier = "challenge@teste.local";

        for _ in 0..4 {
            let decision = record_auth_failure(&db, scope, "establishment", identifier)
                .await
                .unwrap();
            assert!(!decision.required);
            assert_eq!(decision.retry_after_seconds, None);
        }

        let decision = record_auth_failure(&db, scope, "establishment", identifier)
            .await
            .unwrap();
        assert!(decision.required);
        assert!(matches!(decision.retry_after_seconds, Some(seconds) if (899..=900).contains(&seconds)));

        let current = current_auth_challenge(&db, scope, "establishment", identifier)
            .await
            .unwrap();
        assert!(current.required);

        clear_auth_attempts(&db, scope, "establishment", identifier)
            .await
            .unwrap();

        let current = current_auth_challenge(&db, AuthChallengeAction::Login, "establishment", identifier)
            .await
            .unwrap();
        assert!(!current.required);
        assert_eq!(current.retry_after_seconds, None);
    }
}
