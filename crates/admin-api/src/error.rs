use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use karateway_core::{JsonResponse, KaratewayError};
use validator::ValidationErrors;

/// Axum error handler that converts KaratewayError into HTTP responses
pub struct ApiError(pub KaratewayError);

impl From<KaratewayError> for ApiError {
    fn from(err: KaratewayError) -> Self {
        ApiError(err)
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(err: ValidationErrors) -> Self {
        ApiError(KaratewayError::from(err))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = self.0.status_code();
        let error_code = self.0.error_code();
        let message = self.0.to_string();

        let json_response = JsonResponse::<()>::error(status_code, message, Some(error_code));

        let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (status, Json(json_response)).into_response()
    }
}

/// Result type that automatically converts errors to ApiError
pub type ApiResult<T> = Result<T, ApiError>;
