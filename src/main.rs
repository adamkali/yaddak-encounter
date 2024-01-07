mod models;
mod api;
mod traits;

use std::{time::Duration, sync::Arc};

use models::state::YaddakState;
use sqlx::PgPool;
use tokio::{net::TcpListener, signal, sync::Mutex};
use dotenv::dotenv;
use axum::Router;
use tower_http::{trace::TraceLayer, timeout::TimeoutLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// yaddak
use api::user_controller::user_controller;
use traits::migrate;


#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "ub-card-server=debug,tower_http=debug,axum=trace".into()
            }),
            )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let con_str = format!(
        "postgresql://{}:{}@localhost:8050/query",
        std::env::var("DB_USER").unwrap(),
        std::env::var("DB_PASSWORD").unwrap()
    );
    let pool = PgPool::connect(&con_str)
        .await
        .unwrap();

    let _ = migrate(pool.clone()).await;
    let state = Arc::new(Mutex::new(YaddakState { db: pool }));
    let user_router = user_controller(state.clone());

    let app = Router::new()
        .nest("/user", user_router)
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(15)),
        ));

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
