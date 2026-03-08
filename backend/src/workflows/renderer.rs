use crate::db::models::Environment;
use crate::slack::thread_context::NormalizedThread;

use super::loader::WorkflowDefinition;

pub fn render_prompt(
    workflow: Option<&WorkflowDefinition>,
    environment: Option<&Environment>,
    thread: &NormalizedThread,
    workspace_path: &str,
) -> String {
    let mut sections = Vec::new();

    if let Some(workflow) = workflow {
        sections.push(format!(
            "Workflow: {}\nInstructions:\n{}\n",
            workflow.metadata.name, workflow.prompt
        ));
    }

    if let Some(environment) = environment {
        sections.push(format!(
            "Environment: {}\nRepo: {}\nDefault branch: {}\n",
            environment.slug, environment.git_ssh_url, environment.default_branch
        ));
    }

    sections.push(format!("Workspace: {}\n", workspace_path));
    sections.push("Slack thread context:".to_string());
    for message in &thread.messages {
        sections.push(format!("- {}: {}", message.author_label, message.text));
    }

    sections.join("\n")
}
