mod models;
mod api;
mod traits;
mod utilities;

use std::future::Future;
use std::path::PathBuf;
use std::{time::Duration, sync::Arc, net::SocketAddr};
use axum::extract::Host;
use axum::handler::HandlerWithoutStateExt;
use axum::response::Redirect;
use hyper::{Uri, StatusCode};
use models::state::YaddakState;
use tokio::signal;
use dotenv::dotenv;
use axum::{Router, BoxError};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::{trace::TraceLayer, timeout::TimeoutLayer};
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// yaddak
use api::user_controller::{user_controller, user_auth_controller};
use traits::migrate;
use crate::api::api_docs;

#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
}

#[tokio::main]
async fn main() {
    let ports = Ports {
        http: 8010,
        https: 44310,
    };

    dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "yaddak_encounter_server=debug,tower_http=debug,axum=trace".into()
            }),
            )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let db_user = std::env::var("DB_USER").unwrap();
    let db_password = std::env::var("DB_PASSWORD").unwrap();
    let db_url = std::env::var("DB_URL").unwrap();
    let db_name = std::env::var("DB_NAME").unwrap();

    let con_str = format!(
        "postgres://{}:{}@{}/{}",
        db_user,
        db_password,
        db_url,
        db_name
    );

    debug!("migrating");
    let _ = migrate(con_str.clone()).await;
    let state = Arc::new(YaddakState { db: con_str });
    let user_router = user_controller(state.clone());
    let user_auth_router = user_auth_controller(state.clone());

    debug!("creating routes");
    let app = Router::new()
        .merge(api_docs())
        .nest("/user", user_router)
        .nest("/auth/user", user_auth_router)
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(15)),
        ));

    // SSL 
    debug!("Creating SSL");
    let config = RustlsConfig::from_pem_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("yaddak_encounter_certificate.pem"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("yaddak_encounter_private_key.pem")
    )
        .await
        .unwrap();


    let handle  = axum_server::Handle::new();
    let shutdown_future = shutdown_signal(handle.clone());

    // Run the server with graceful shutdown
    let addr = SocketAddr::from(([127,0,0,1], ports.https));
    debug!("Listening:\t{addr}");

    axum_server::bind_rustls(addr, config)
        .handle(handle)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn shutdown_signal(handle: axum_server::Handle) {
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

    tracing::info!("Received termination signal shutting down");
    handle.graceful_shutdown(Some(Duration::from_secs(10))); // 10 secs is how long docker will wait
                                                             // to force shutdown
}

async fn redirect_http_to_https<F>(ports: Ports, signal: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    fn make_https(host: String, uri: Uri, ports: Ports) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&ports.http.to_string(), &ports.https.to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, ports) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], ports.http));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {addr}");
    axum::serve(listener, redirect.into_make_service())
        .with_graceful_shutdown(signal)
        .await
        .unwrap();
}
