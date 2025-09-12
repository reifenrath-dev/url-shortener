mod routes;
mod utils;
mod auth;

use std::error::Error;
use axum::{middleware, Router};
use axum::routing::{get, patch, post};
use axum_prometheus::PrometheusMetricLayer;
use dotenv::dotenv;
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing::debug;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::auth::auth;
use crate::routes::{create_link, get_link_statistics, health, update_link};
use crate::routes::redirect;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "url_shortener=debug".into())
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&db_url)
        .await
        .unwrap_or_else(|_| panic!("Failed to create Postgres connection pool! URL: {}", db_url));

    sqlx::migrate!().run(&db).await?;

    let (prometheus_layer, metrics_handle) = PrometheusMetricLayer::pair();

    let app = Router::new()
        .route("/api/create", post(create_link))
        .route("/api/statistics/{*id}", get(get_link_statistics))
        .route("/api/metrics", get(|| async move {metrics_handle.render()}))
        .route_layer(middleware::from_fn_with_state(db.clone(), auth))
        .route("/api/health", get(health))
        .layer(TraceLayer::new_for_http())
        .layer(prometheus_layer)
        .fallback_service(wildcard_router(db.clone()))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Could not initialize tcp listener");

    debug!(
        "Listening on {}",
        listener
            .local_addr()
            .expect("Could not convert address to local address")
    );

    axum::serve(listener, app)
        .await
        .expect("Could not successfully create server");

    Ok(())
}

fn wildcard_router(db: Pool<Postgres>) -> Router {
    Router::new()
        .route("/{*id}", get(redirect))
        .route("/api/{*id}", patch(update_link).route_layer(middleware::from_fn_with_state(db.clone(), auth)))
        .with_state(db)
}