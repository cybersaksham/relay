use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub id: String,
    pub name: String,
    pub scope: String,
    #[serde(default)]
    pub environment_slug: Option<String>,
    #[serde(default)]
    pub trigger_phrases: Vec<String>,
    #[serde(default)]
    pub default_environment: Option<String>,
    #[serde(default)]
    pub instructions: Vec<String>,
    #[serde(default = "default_response_mode")]
    pub response_mode: String,
}

#[derive(Debug, Clone)]
pub struct WorkflowDefinition {
    pub metadata: WorkflowMetadata,
    pub prompt: String,
    pub root_dir: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct WorkflowRegistry {
    workflows: HashMap<String, WorkflowDefinition>,
}

fn default_response_mode() -> String {
    "reply".to_string()
}

impl WorkflowRegistry {
    pub fn load(root: &Path) -> Result<Self> {
        let mut workflows = HashMap::new();
        if !root.exists() {
            return Ok(Self { workflows });
        }

        for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
            if entry.file_name() != "workflow.yaml" {
                continue;
            }

            let workflow_path = entry.path();
            let prompt_path = workflow_path
                .parent()
                .expect("workflow dir")
                .join("prompt.md");
            let metadata: WorkflowMetadata = serde_yaml::from_str(
                &fs::read_to_string(workflow_path)
                    .with_context(|| format!("failed to read {}", workflow_path.display()))?,
            )
            .with_context(|| format!("failed to parse {}", workflow_path.display()))?;
            let prompt = fs::read_to_string(&prompt_path)
                .with_context(|| format!("failed to read {}", prompt_path.display()))?;

            workflows.insert(
                metadata.id.clone(),
                WorkflowDefinition {
                    metadata,
                    prompt,
                    root_dir: workflow_path.parent().unwrap().to_path_buf(),
                },
            );
        }

        Ok(Self { workflows })
    }

    pub fn all(&self) -> Vec<WorkflowDefinition> {
        self.workflows.values().cloned().collect()
    }

    pub fn get(&self, id: &str) -> Option<WorkflowDefinition> {
        self.workflows.get(id).cloned()
    }

    #[cfg(test)]
    pub fn from_workflows(workflows: HashMap<String, WorkflowDefinition>) -> Self {
        Self { workflows }
    }
}
