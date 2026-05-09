use crate::utils::internal_error;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use metrics::counter;
use sha3::{Digest, Sha3_256};
use sqlx::PgPool;
use hex;

struct Setting {
    #[allow(dead_code)]
    id: String,
    encrypted_global_api_key: String,
}

/// This middleware function validates that incoming requests contain a valid
/// API key in the "x-api-key" header. It compares the provided key against
/// a stored encrypted API key from the database.
pub async fn auth(
    State(pool): State<PgPool>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let labels = [("uri", format!("{}!", req.uri()))];

    let api_key = req
        .headers()
        .get("x-api-key")
        .map(|value| value.to_str().unwrap_or_default())
        .ok_or_else(|| {
            tracing::error!("Unauthorized call to API: No key header received");
            counter!("unauthenticated_calls_count", &labels).increment(1);

            (StatusCode::UNAUTHORIZED, "Unauthorized".into())
        })?;

    let fetch_setting_timeout = tokio::time::Duration::from_millis(300);

    let setting = tokio::time::timeout(
        fetch_setting_timeout,
        sqlx::query_as!(
            Setting,
            "select id, encrypted_global_api_key from settings where id = $1",
            "DEFAULT_SETTINGS"
        )
        .fetch_one(&pool),
    )
    .await
    .map_err(internal_error)?
    .map_err(internal_error)?;

    if !is_valid_api_key(&setting.encrypted_global_api_key, api_key) {
        tracing::error!("Unauthorized call to API: Incorrect key supplied");
        counter!("unauthenticated_calls_count", &labels).increment(1);

        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".into()));
    }

    Ok(next.run(req).await)
}

/// Validates an API key by comparing it with its SHA3-256 hashed and hex-encoded version.
///
/// # Arguments
///
/// * `encrypted_api_key` - A string slice containing the expected SHA3-256 hex-encoded API key
/// * `api_key` - A string slice containing the API key to validate
///
/// # Returns
///
/// * `bool` - Returns `true` if the provided API key matches the encrypted version,
///            `false` otherwise
fn is_valid_api_key(encrypted_api_key: &str, api_key: &str) -> bool {
    let mut hasher = Sha3_256::new();
    hasher.update(api_key.as_bytes());
    let provided_api_key = hasher.finalize();
    let hexed_provided_api_key = hex::encode(provided_api_key);

    encrypted_api_key == hexed_provided_api_key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_valid_api_key_true() {
        let mut hasher = Sha3_256::new();
        hasher.update("abc".as_bytes());
        let encrypted_api_key = hasher.finalize();
        let hex = hex::encode(encrypted_api_key);

        let result = is_valid_api_key(&hex, "abc");

        assert!(result);
    }

    #[test]
    fn is_valid_api_key_false() {
        let mut hasher = Sha3_256::new();
        hasher.update("abc".as_bytes());
        let encrypted_api_key = hasher.finalize();
        let hex = hex::encode(encrypted_api_key);

        let result = is_valid_api_key(&hex, "abd");

        assert!(!result);
    }
}