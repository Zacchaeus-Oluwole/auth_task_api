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

pub async fn get_task(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
) -> AppResult<Json<Task>>{
    let task = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasts WHERE id = $1 AND user_id = $2"
    )
    .bind(id)
    .bind(user.user_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(
        || AppError::NotFound { 
            resource: "Task".to_string(), 
            id: id.to_string(), 
        }
    )?;

    Ok(Json(task))
}


pub async fn update_task(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdateTask>,
) -> AppResult<Json<Task>> {
    let task =sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE id = $1 AND user_id = $2"
    )
    .bind(id)
    .bind(user.user_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(
        || AppError::NotFound { 
            resource: "Task".to_string(), 
            id: id.to_string() 
        }
    )?;

    let title = payload.title.as_ref().unwrap_or(&task.title);
    let description = payload.description.as_ref().or(task.description.as_ref());
    let status = payload.status.as_ref().unwrap_or(&task.status);

    let updated = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET title = $1, description = $2, status = $3, updated_at = NOW() WHERE id = $4 RETURNING *"
    )
    .bind(title)
    .bind(description)
    .bind(status)
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(updated))
}


pub async fn delete_task(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
) -> AppResult<axum::http::StatusCode> {
    let deleted = sqlx::query(
        "DELETE FROM tasks WHERE id = $1 AND user_id = $2"
    )
    .bind(id)
    .bind(user.user_id)
    .execute(&pool)
    .await?;

    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound { resource: "Task".to_string(), id: id.to_string(), });
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}

