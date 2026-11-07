//! HTTP errors: every failure becomes `{error, code}` with one status per `AppError` group.

use std::sync::OnceLock;

use arrhabon_app::AppError;
use axum::Json;
use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::extract::{FromRequest, FromRequestParts};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, PartialEq, Eq)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
    pub signature: Option<String>,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &str, message: impl Into<String>) -> Self {
        ApiError {
            status,
            code: code.into(),
            message: scrub(&message.into()),
            signature: None,
        }
    }

    pub fn already_mirrored(signature: &str) -> Self {
        ApiError {
            status: StatusCode::CONFLICT,
            code: "already_mirrored".into(),
            message: "swap has already been mirrored".into(),
            signature: Some(signature.into()),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "invalid_input", message)
    }

    pub fn chain_not_initialised() -> Self {
        Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "chain_not_initialised",
            "the program is not initialised on this cluster yet",
        )
    }
}

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        use StatusCode as S;
        let (status, code) = match &e {
            AppError::Forbidden(d) => (S::FORBIDDEN, format!("{d:?}")),
            AppError::Conflict(d) => (S::CONFLICT, format!("{d:?}")),
            AppError::Invalid(d) => (S::BAD_REQUEST, format!("{d:?}")),
            AppError::InvalidInput(_) => (S::BAD_REQUEST, "invalid_input".into()),
            AppError::VaultNotFound(_) => (S::NOT_FOUND, "vault_not_found".into()),
            AppError::UniversityNotRegistered(_) => {
                (S::BAD_REQUEST, "university_not_registered".into())
            }
            AppError::AttestorNotRegistered(_) => {
                (S::BAD_REQUEST, "attestor_not_registered".into())
            }
            AppError::AlreadyApplied => (S::CONFLICT, "already_applied".into()),
            AppError::AlreadyFunded => (S::CONFLICT, "already_funded".into()),
            AppError::SwapPending => (S::CONFLICT, "swap_pending".into()),
            AppError::SwapNotSettled => (S::CONFLICT, "swap_not_settled".into()),
            AppError::Rejected(_) => (S::BAD_REQUEST, "rejected".into()),
            AppError::Unavailable(detail) => {
                // The detail can name upstream hosts; it goes to logs (scrubbed), not clients.
                tracing::warn!(detail = %scrub(detail), "upstream unavailable");
                return Self::new(
                    S::SERVICE_UNAVAILABLE,
                    "unavailable",
                    "upstream unavailable",
                );
            }
        };
        Self::new(status, &code, e.to_string())
    }
}

impl From<JsonRejection> for ApiError {
    fn from(r: JsonRejection) -> Self {
        let status = match r.status() {
            StatusCode::PAYLOAD_TOO_LARGE => StatusCode::PAYLOAD_TOO_LARGE,
            _ => StatusCode::BAD_REQUEST,
        };
        Self::new(status, "invalid_body", r.body_text())
    }
}

impl From<QueryRejection> for ApiError {
    fn from(r: QueryRejection) -> Self {
        Self::bad_request(r.body_text())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut body = json!({ "error": self.message, "code": self.code });
        if let Some(sig) = self.signature {
            body["signature"] = json!(sig);
        }
        (self.status, Json(body)).into_response()
    }
}

/// `axum::Json` whose rejection is an `ApiError`, so bad bodies get the same JSON shape.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct Body<T>(pub T);

/// `axum::extract::Query` with an `ApiError` rejection.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(ApiError))]
pub struct Query<T>(pub T);

static SECRETS: OnceLock<Vec<String>> = OnceLock::new();

/// Registers strings (an RPC URL carrying an API key) that `scrub` removes; first call wins.
pub fn register_secrets(secrets: Vec<String>) {
    let _ = SECRETS.set(secrets.into_iter().filter(|s| !s.is_empty()).collect());
}

/// Removes registered secrets from text bound for clients or logs.
pub fn scrub(text: &str) -> String {
    SECRETS
        .get()
        .into_iter()
        .flatten()
        .fold(text.to_owned(), |t, s| t.replace(s.as_str(), "<redacted>"))
}
