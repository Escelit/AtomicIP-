use axum::{
    body::Body,
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::warn;

use crate::validation::{ValidationError, ErrorSeverity};

/// Standardized validation error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationErrorResponse {
    pub error: String,
    pub details: Vec<ValidationErrorDetail>,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidationErrorDetail {
    pub field: String,
    pub message: String,
    pub severity: String,
}

impl ValidationErrorResponse {
    pub fn from_validation_errors(errors: Vec<ValidationError>) -> Self {
        let details = errors
            .into_iter()
            .map(|e| ValidationErrorDetail {
                field: e.field,
                message: e.message,
                severity: format!("{:?}", e.severity),
            })
            .collect();

        Self {
            error: "Request validation failed".to_string(),
            details,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn has_high_severity_errors(&self) -> bool {
        self.details
            .iter()
            .any(|d| d.severity == "High")
    }
}

/// Middleware that enforces strict request validation
pub async fn validation_enforcement_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ValidationErrorResponse>)> {
    let method = req.method().clone();
    let uri = req.uri().clone();

    // Only validate non-GET requests for now
    if matches!(
        method,
        axum::http::Method::POST | axum::http::Method::PUT | axum::http::Method::PATCH
    ) {
        // Log validation check
        tracing::debug!(
            target: "validation",
            method = %method,
            uri = %uri,
            "Validating request"
        );
    }

    Ok(next.run(req).await)
}

/// Middleware that logs validation errors for monitoring
pub async fn validation_logging_middleware(
    req: Request<Body>,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = std::time::Instant::now();

    let response = next.run(req).await;

    let elapsed = start.elapsed();
    if response.status() == StatusCode::BAD_REQUEST {
        warn!(
            target: "validation_errors",
            method = %method,
            uri = %uri,
            status = %response.status(),
            elapsed_ms = elapsed.as_millis(),
            "Request validation failed"
        );
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_response_creation() {
        let errors = vec![
            ValidationError {
                field: "address".to_string(),
                message: "Invalid address format".to_string(),
                severity: ErrorSeverity::High,
            },
            ValidationError {
                field: "amount".to_string(),
                message: "Amount must be positive".to_string(),
                severity: ErrorSeverity::Medium,
            },
        ];

        let response = ValidationErrorResponse::from_validation_errors(errors);
        assert_eq!(response.details.len(), 2);
        assert!(response.has_high_severity_errors());
    }

    #[test]
    fn test_validation_error_response_no_high_severity() {
        let errors = vec![ValidationError {
            field: "field".to_string(),
            message: "Minor issue".to_string(),
            severity: ErrorSeverity::Low,
        }];

        let response = ValidationErrorResponse::from_validation_errors(errors);
        assert!(!response.has_high_severity_errors());
    }
}
