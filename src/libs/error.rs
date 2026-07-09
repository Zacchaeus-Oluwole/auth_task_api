use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json
};

use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    ValidationError(validator::ValidationErrors),
    NotFound{ resource: String, id: String},
    Unauthorized(String),
    Forbidden(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            AppError::DatabaseError(err) => {
                tracing::error!("Database error: {err}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({
                        "error": {
                            "code": "DATABASE_ERROR",
                            "messsage": "A database error occurred"
                        }
                    })
                )
            }

            AppError::ValidationError(errors) => {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    json!({
                        "error": {
                            "code": "VALIDATION_ERROR",
                            "message": "Validation failed",
                            "details": errors.field_errors()
                        }
                    }),
                )
            }
            AppError::NotFound { resource, id } => {
                (
                    StatusCode::NOT_FOUND,
                    json!({
                        "error": {
                            "code": "NOT_FOUND",
                            "message": format!("{} with id {} not found", resource, id)
                        }
                    }),
                )
            }
            AppError::Unauthorized(msg) => {
                (
                    StatusCode::UNAUTHORIZED,
                    json!({
                        "error": {
                            "code": "UNAUTHORIZED",
                            "message": msg
                        }
                    }),
                )
            }
            AppError::Forbidden(msg) => {
                (
                    StatusCode::FORBIDDEN,
                    json!({
                        "error": {
                            "code": "FORBIDDEN",
                            "message": msg
                        }
                    }),
                )
            }
        };

        (status, Json(error_response)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        AppError::DatabaseError(value)
    }   
}

impl From<validator::ValidationErrors> for AppError {
    fn from(value: validator::ValidationErrors) -> Self {
        AppError::ValidationError(value)
    }    
}

pub type AppResult<T> = Result<T, AppError>;