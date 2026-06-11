use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("nao autenticado")]
    Unauthorized,
    #[error("acesso negado")]
    Forbidden,
    #[error("registro nao encontrado")]
    NotFound,
    #[error("dados invalidos: {0}")]
    BadRequest(String),
    #[error("validacao humana necessaria")]
    ChallengeRequired(ChallengeRequiredResponse),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let user_facing_error = self.user_facing_error();
        let status = match &user_facing_error {
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden => StatusCode::FORBIDDEN,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::ChallengeRequired(_) => StatusCode::TOO_MANY_REQUESTS,
            ApiError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = match &user_facing_error {
            ApiError::ChallengeRequired(details) => Json(json!({
                "error": user_facing_error.to_string(),
                "status": status.as_u16(),
                "challenge_required": true,
                "captcha_provider": details.captcha_provider,
                "captcha_site_key": details.captcha_site_key,
                "action": details.action,
                "retry_after_seconds": details.retry_after_seconds,
            })),
            _ => Json(json!({
                "error": user_facing_error.to_string(),
                "status": status.as_u16()
            })),
        };
        (status, body).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    fn user_facing_error(self) -> Self {
        match self {
            ApiError::Database(error) if is_unique_constraint_error(&error) => {
                ApiError::BadRequest("registro ja cadastrado".into())
            }
            other => other,
        }
    }

    pub fn challenge_required(action: impl Into<String>, retry_after_seconds: i64) -> Self {
        Self::ChallengeRequired(ChallengeRequiredResponse {
            challenge_required: true,
            captcha_provider: "hcaptcha",
            captcha_site_key: captcha_site_key_from_env(),
            action: action.into(),
            retry_after_seconds,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChallengeRequiredResponse {
    pub challenge_required: bool,
    pub captcha_provider: &'static str,
    pub captcha_site_key: String,
    pub action: String,
    pub retry_after_seconds: i64,
}

fn captcha_site_key_from_env() -> String {
    std::env::var("HCAPTCHA_SITE_KEY")
        .unwrap_or_else(|_| "10000000-ffff-ffff-ffff-000000000001".to_string())
}

fn is_unique_constraint_error(error: &sqlx::Error) -> bool {
    let sqlx::Error::Database(database_error) = error else {
        return false;
    };
    database_error
        .code()
        .is_some_and(|code| code == "2067" || code == "1555")
        || database_error
            .message()
            .to_ascii_lowercase()
            .contains("unique constraint failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body, response::Response};
    use serde_json::Value;
    use sqlx::{Executor, SqlitePool};

    async fn response_body(response: Response) -> Value {
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn sqlite_unique_constraint_returns_bad_request() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        pool.execute("create table people (email text unique)")
            .await
            .unwrap();
        sqlx::query("insert into people (email) values ('a@teste.local')")
            .execute(&pool)
            .await
            .unwrap();
        let error = sqlx::query("insert into people (email) values ('a@teste.local')")
            .execute(&pool)
            .await
            .unwrap_err();

        let response = ApiError::from(error).into_response();
        let status = response.status();
        let body = response_body(response).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["status"], 400);
        assert!(body["error"].as_str().unwrap().contains("ja cadastrado"));
    }
}
