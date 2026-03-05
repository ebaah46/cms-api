use axum::{
    Json, async_trait,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use serde_json::json;

/// Custom JSON extractor that returns consistent JSON error responses
pub struct AppJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for AppJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = JsonError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(AppJson(value)),
            Err(rejection) => Err(JsonError(rejection)),
        }
    }
}

pub struct JsonError(JsonRejection);

impl IntoResponse for JsonError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            JsonRejection::JsonDataError(err) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Invalid JSON data: {}", err.body_text()),
            ),
            JsonRejection::JsonSyntaxError(err) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid JSON syntax: {}", err.body_text()),
            ),
            JsonRejection::MissingJsonContentType(_) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Missing Content-Type: application/json header".to_string(),
            ),
            JsonRejection::BytesRejection(_) => (
                StatusCode::BAD_REQUEST,
                "Failed to read request body".to_string(),
            ),
            _ => (StatusCode::BAD_REQUEST, "Invalid request body".to_string()),
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
