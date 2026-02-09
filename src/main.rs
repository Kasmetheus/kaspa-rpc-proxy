mod auth;
mod client;
mod error;
mod handlers;
mod metrics;
mod models;
mod websocket;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kaspa_rpc_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Starting Kaspa RPC Service");

    // Load configuration
    let config = load_config()?;
    
    // Initialize Kaspa gRPC client
    let kaspa_client = client::KaspaClient::new(&config.kaspa_rpc_url).await?;
    tracing::info!("✓ Connected to Kaspa node at {}", config.kaspa_rpc_url);

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(handlers::health_check))
        .route("/metrics", get(handlers::metrics))
        
        // Core RPC endpoints
        .route("/rpc/getBlock", post(handlers::get_block))
        .route("/rpc/submitTransaction", post(handlers::submit_transaction))
        .route("/rpc/getDAGTips", post(handlers::get_dag_tips))
        
        // WebSocket for subscriptions
        .route("/ws/subscribeUTXO", get(websocket::subscribe_utxo))
        
        // Middleware
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            kaspa_client: std::sync::Arc::new(kaspa_client),
            jwt_secret: config.jwt_secret.clone(),
        });

    // Start server
    let addr: SocketAddr = config.bind_address.parse()?;
    tracing::info!("🌐 Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    kaspa_client: std::sync::Arc<client::KaspaClient>,
    jwt_secret: String,
}

#[derive(Debug, serde::Deserialize)]
struct Config {
    kaspa_rpc_url: String,
    bind_address: String,
    jwt_secret: String,
}

fn load_config() -> anyhow::Result<Config> {
    dotenv::dotenv().ok();
    
    Ok(Config {
        kaspa_rpc_url: std::env::var("KASPA_RPC_URL")
            .unwrap_or_else(|_| "http://localhost:16110".to_string()),
        bind_address: std::env::var("BIND_ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
        jwt_secret: std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "CHANGE_ME_IN_PRODUCTION".to_string()),
    })
}
