mod libs;

use axum::{
    extract::{FromRequestParts, Request},
    middleware::Next,
    response::Response,
    routing::{get, post, put, delete},
    Router,
    http::{StatusCode, header::AUTHORIZATION, request::Parts}
};
use uuid::Uuid;

use crate::libs::{db, handlers::{auth, tasks}, models::AuthenticatedUser};
use sqlx::PgPool;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::env;



async fn auth_middleware(
    mut request: Request,
    next: Next
) -> Result<Response, StatusCode> {
    let token = request.headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| {
            if h.starts_with("Bearer ") {
                Some(&h[7..])
            } else {
                None
            }
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let secret = env::var("JWT_SECRET")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let claims = libs::auth::verify_token(token, &secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(AuthenticatedUser {
        user_id,
        role: claims.role,
    });

    Ok(next.run(request).await)
}

impl <S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync + Clone
     {
        type Rejection = StatusCode;

        async fn from_request_parts(
            parts: &mut Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection>
        {
            parts.extensions
                .get::<AuthenticatedUser>()
                .cloned()
                .ok_or(StatusCode::UNAUTHORIZED)
        }
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "auth_task_api=debug".into())
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    init_tracing();

    let pool = libs::db::create_pool().await?;

    let public = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login));

    let protected = Router::new()
        .route("/tasks", post(tasks::create_task).get(tasks::list_tasks))
        .route("/tasks/{id}", get(tasks::get_task)
            .put(tasks::update_task)
            .delete(tasks::delete_task))
        .route_layer(axum::middleware::from_fn(auth_middleware)); // guards only these

    let app = Router::new()
        .merge(public)
        .merge(protected)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(pool);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;
    
    Ok(())
}
