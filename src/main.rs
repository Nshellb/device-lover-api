use std::time::Duration;

use device_lover_api::build_app;
use device_lover_api::config::Settings;
use device_lover_api::state::AppState;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    if let Err(error) = run().await {
        tracing::error!(error = ?error, "application failed");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::load()?;
    let pool = PgPoolOptions::new().connect(&settings.database_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState::new(settings.clone(), pool);
    let app = build_app(state, REQUEST_TIMEOUT);

    let listener = tokio::net::TcpListener::bind(settings.socket_addr()).await?;
    tracing::info!(address = %listener.local_addr()?, "listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await?;
        Ok::<_, std::io::Error>("Ctrl+C")
    };

    #[cfg(unix)]
    let terminate = async {
        let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        signal.recv().await;
        Ok::<_, std::io::Error>("SIGTERM")
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<Result<&'static str, std::io::Error>>();

    let signal = tokio::select! {
        result = ctrl_c => result,
        result = terminate => result,
    };

    match signal {
        Ok(signal) => tracing::info!(signal, "received shutdown signal"),
        Err(error) => tracing::error!(error = ?error, "failed to install shutdown signal handler"),
    }
}
