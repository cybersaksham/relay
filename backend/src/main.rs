use std::sync::Arc;

use anyhow::Context;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info};

use relay_backend::api;
use relay_backend::app_state::AppState;
use relay_backend::config::Config;
use relay_backend::db::Database;
use relay_backend::environments::service::EnvironmentService;
use relay_backend::policies::evaluator::PolicyEngine;
use relay_backend::runner::codex_cli::CodexCliRunner;
use relay_backend::runner::Runner;
use relay_backend::slack;
use relay_backend::slack::web_api::SlackWebClient;
use relay_backend::tasks::session_service::SessionService;
use relay_backend::utils;
use relay_backend::workflows::loader::WorkflowRegistry;
use relay_backend::workspaces::session_workspace::WorkspaceManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    utils::tracing::init()?;

    let config = Arc::new(Config::from_env()?);
    tokio::fs::create_dir_all(&config.paths.relay_home).await?;
    tokio::fs::create_dir_all(&config.paths.sources_dir).await?;
    tokio::fs::create_dir_all(&config.paths.workspaces_dir).await?;

    let db = Database::connect(&config.database.url).await?;
    db.migrate().await?;

    let policy_engine = Arc::new(PolicyEngine::load(
        &config.paths.policies_dir,
        config.authorization.master_slack_user_ids.clone(),
    )?);
    let workflow_registry = Arc::new(WorkflowRegistry::load(&config.paths.workflows_dir)?);
    let workspace_manager = Arc::new(WorkspaceManager::new(config.clone()));
    let environment_service = Arc::new(EnvironmentService::new(
        db.pool().clone(),
        workspace_manager.clone(),
    ));
    let slack_client = Arc::new(SlackWebClient::new(config.clone()));
    let session_service = Arc::new(SessionService::new(db.pool().clone()));
    let runner: Arc<dyn Runner> = Arc::new(CodexCliRunner::new(config.clone(), db.pool().clone()));

    let state = Arc::new(AppState::new(
        config.clone(),
        db.pool().clone(),
        slack_client,
        policy_engine,
        workflow_registry,
        environment_service,
        workspace_manager,
        session_service,
        runner,
    ));

    let router: Router = api::router(state.clone())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let slack_state = state.clone();
    tokio::spawn(async move {
        if let Err(error) = slack::socket_mode::run_socket_mode(slack_state).await {
            error!(?error, "socket mode worker exited");
        }
    });

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind to {addr}"))?;

    info!("relay backend listening on {}", addr);
    axum::serve(listener, router).await?;
    Ok(())
}
