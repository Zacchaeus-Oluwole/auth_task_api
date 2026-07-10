use std::env;

use axum::{extract::State, Json};                                                                                                        
use sqlx::PgPool;
use validator::Validate;
use crate::libs::auth::{hash_password, verify_password, create_token };
use crate::libs::error::{AppError, AppResult};
use crate::libs::models::{RegisterRequest, LoginRequest, AuthResponse, User};

pub async fn register (
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    payload.validate()?;
    
    let existing = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_one(&pool)
    .await?;

    if existing {
        return Err(AppError::ValidationError(
            validator::ValidationErrors::new()
        ));
    }

    let password_hash = hash_password(&payload.password)
        .map_err(|_| AppError::Unauthorized("Password hashing failed".to_string()))?;

    let user = sqlx::query_as::<_, User> (
            "INSERT INTO users (email, password_hash, role) VALUES ($1, $2, $3) RETURNING *"
        )
        .bind(&payload.email)
        .bind(&password_hash)
        .fetch_one(&pool)
        .await?;




    let secret = env::var("JWT_SECRET")
        .map_err(
            |_| AppError::Unauthorized("JWT secret not configured".to_string())
        )?;

    let token = create_token(user.id, &user.role, &secret)
        .map_err(|_| AppError::Unauthorized("Token creation failed".to_string()))?;


    Ok(
        Json(
            AuthResponse { 
                token, 
                user_id: user.id, 
                role: user.role
            }
        )
    )
}

pub async fn login (
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    if !verify_password(&payload.password, &user.password_hash).map_err(|_| AppError::Unauthorized("Password verification failed".to_string()))? {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let secret = env::var("JWT_SECRET")
        .map_err(|_| AppError::Unauthorized("JWT secret not configured".to_string()))?;

    let token = create_token(user.id, &user.role, &secret).map_err(|_| AppError::Unauthorized("Token creation failed".to_string()))?;


    Ok(
        Json(
            AuthResponse { 
                token, 
                user_id: user.id, 
                role: user.role 
            }
        )
    )
}