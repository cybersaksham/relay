use std::sync::Arc;

use sqlx::SqlitePool;

use crate::config::SharedConfig;
use crate::environments::service::EnvironmentService;
use crate::policies::evaluator::PolicyEngine;
use crate::runner::Runner;
use crate::slack::web_api::SlackWebClient;
use crate::tasks::session_service::SessionService;
use crate::terminal::TerminalManager;
use crate::workflows::loader::WorkflowRegistry;
use crate::workspaces::session_workspace::WorkspaceManager;

#[derive(Clone)]
pub struct AppState {
    pub config: SharedConfig,
    pub db: SqlitePool,
    pub slack: Arc<SlackWebClient>,
    pub policies: Arc<PolicyEngine>,
    pub workflows: Arc<WorkflowRegistry>,
    pub environments: Arc<EnvironmentService>,
    pub workspaces: Arc<WorkspaceManager>,
    pub sessions: Arc<SessionService>,
    pub terminals: Arc<TerminalManager>,
    pub runner: Arc<dyn Runner>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: SharedConfig,
        db: SqlitePool,
        slack: Arc<SlackWebClient>,
        policies: Arc<PolicyEngine>,
        workflows: Arc<WorkflowRegistry>,
        environments: Arc<EnvironmentService>,
        workspaces: Arc<WorkspaceManager>,
        sessions: Arc<SessionService>,
        terminals: Arc<TerminalManager>,
        runner: Arc<dyn Runner>,
    ) -> Self {
        Self {
            config,
            db,
            slack,
            policies,
            workflows,
            environments,
            workspaces,
            sessions,
            terminals,
            runner,
        }
    }
}
