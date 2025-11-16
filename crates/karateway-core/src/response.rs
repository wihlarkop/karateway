use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Metadata for paginated responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MetaResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_data: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pages: Option<u32>,
}

impl MetaResponse {
    pub fn new(page: u32, limit: u32, total_data: u64) -> Self {
        let total_pages = if limit > 0 {
            ((total_data as f64) / (limit as f64)).ceil() as u32
        } else {
            0
        };

        Self {
            page: Some(page),
            limit: Some(limit),
            total_data: Some(total_data),
            total_pages: Some(total_pages),
        }
    }

    pub fn empty() -> Self {
        Self {
            page: None,
            limit: None,
            total_data: Some(0),
            total_pages: None,
        }
    }
}

/// Standard JSON response wrapper
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JsonResponse<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    pub success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<MetaResponse>,

    pub status_code: u16,

    pub timestamp: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

impl<T> JsonResponse<T> {
    /// Create a successful response with data
    pub fn success(data: T) -> Self {
        Self {
            data: Some(data),
            message: None,
            success: true,
            meta: None,
            status_code: 200,
            timestamp: Utc::now(),
            error_code: None,
        }
    }

    /// Create a successful response with data and custom message
    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            data: Some(data),
            message: Some(message.into()),
            success: true,
            meta: None,
            status_code: 200,
            timestamp: Utc::now(),
            error_code: None,
        }
    }

    /// Create a successful paginated response
    pub fn success_paginated(data: T, meta: MetaResponse) -> Self {
        Self {
            data: Some(data),
            message: None,
            success: true,
            meta: Some(meta),
            status_code: 200,
            timestamp: Utc::now(),
            error_code: None,
        }
    }

    /// Create a created response (201)
    pub fn created(data: T, message: impl Into<String>) -> Self {
        Self {
            data: Some(data),
            message: Some(message.into()),
            success: true,
            meta: None,
            status_code: 201,
            timestamp: Utc::now(),
            error_code: None,
        }
    }

    /// Create an accepted response (202)
    pub fn accepted(message: impl Into<String>) -> Self
    where
        T: Default,
    {
        Self {
            data: None,
            message: Some(message.into()),
            success: true,
            meta: None,
            status_code: 202,
            timestamp: Utc::now(),
            error_code: None,
        }
    }
}

impl JsonResponse<()> {
    /// Create a no-content success response (204)
    pub fn no_content() -> Self {
        Self {
            data: None,
            message: Some("Operation completed successfully".to_string()),
            success: true,
            meta: None,
            status_code: 204,
            timestamp: Utc::now(),
            error_code: None,
        }
    }

    /// Create an error response
    pub fn error(status_code: u16, message: impl Into<String>, error_code: Option<String>) -> Self {
        Self {
            data: None,
            message: Some(message.into()),
            success: false,
            meta: None,
            status_code,
            timestamp: Utc::now(),
            error_code,
        }
    }

    /// Create a bad request error (400)
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::error(400, message, Some("BAD_REQUEST".to_string()))
    }

    /// Create an unauthorized error (401)
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::error(401, message, Some("UNAUTHORIZED".to_string()))
    }

    /// Create a forbidden error (403)
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::error(403, message, Some("FORBIDDEN".to_string()))
    }

    /// Create a not found error (404)
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::error(404, message, Some("NOT_FOUND".to_string()))
    }

    /// Create a conflict error (409)
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::error(409, message, Some("CONFLICT".to_string()))
    }

    /// Create an internal server error (500)
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::error(500, message, Some("INTERNAL_ERROR".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_response() {
        let response = JsonResponse::success("test data");
        assert!(response.success);
        assert_eq!(response.status_code, 200);
        assert_eq!(response.data, Some("test data"));
    }

    #[test]
    fn test_paginated_response() {
        let meta = MetaResponse::new(1, 10, 100);
        assert_eq!(meta.page, Some(1));
        assert_eq!(meta.limit, Some(10));
        assert_eq!(meta.total_data, Some(100));
        assert_eq!(meta.total_pages, Some(10));
    }

    #[test]
    fn test_error_response() {
        let response = JsonResponse::<()>::not_found("Resource not found");
        assert!(!response.success);
        assert_eq!(response.status_code, 404);
        assert_eq!(response.error_code, Some("NOT_FOUND".to_string()));
    }
}
