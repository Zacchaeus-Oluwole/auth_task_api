use axum::{
    extract::{Path, State},
    Json,
};

use uuid::Uuid;
use sqlx::PgPool;
use validator::Validate;
use crate::libs::error::{AppError, AppResult};
use crate::libs::models::{Task, CreateTask, UpdateTask, AuthenticatedUser};

pub async fn create_task(
    State(pool): State<PgPool>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateTask>,
) -> AppResult<Json<Task>> {
    payload.validate()?;

    let status = payload.status.unwrap_or_else(|| "todp".to_string());

    let task = sqlx::query_as::<_, Task>(
        "INSERT INTO tasks (title, description, user_id, status) VALUES ($1,$2,$3,$4) RETURNING *"
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(user.user_id)
    .bind(&status)
    .fetch_one(&pool)
    .await?;

    Ok(Json(task))
}


