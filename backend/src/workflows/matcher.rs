use crate::db::models::Environment;

use super::loader::{WorkflowDefinition, WorkflowRegistry};

pub fn match_workflow(
    registry: &WorkflowRegistry,
    prompt: &str,
    environment: Option<&Environment>,
) -> Option<WorkflowDefinition> {
    let prompt_lower = prompt.to_lowercase();

    if let Some(explicit_id) = prompt_lower
        .split_whitespace()
        .find_map(|token| token.strip_prefix("workflow:"))
    {
        if let Some(workflow) = registry.get(explicit_id) {
            return Some(workflow);
        }
    }

    let mut candidates = Vec::new();
    for workflow in registry.all() {
        let mut score = 0_i32;

        if let Some(environment) = environment {
            if workflow.metadata.environment_slug.as_deref() == Some(environment.slug.as_str()) {
                score += 5;
            } else if workflow.metadata.scope == "environment" {
                continue;
            }
        } else if workflow.metadata.scope == "environment" {
            continue;
        }

        for phrase in &workflow.metadata.trigger_phrases {
            if phrase_matches(&prompt_lower, phrase) {
                score += 3;
            }
        }

        if prompt_lower.contains(&workflow.metadata.id.to_lowercase()) {
            score += 2;
        }

        if score > 0 {
            candidates.push((score, workflow));
        }
    }

    candidates.sort_by(|left, right| right.0.cmp(&left.0));
    match candidates.as_slice() {
        [] => None,
        [(score_a, workflow_a)] => {
            let _ = score_a;
            Some(workflow_a.clone())
        }
        [(score_a, workflow_a), (score_b, _)] if score_a > score_b => Some(workflow_a.clone()),
        _ => None,
    }
}

fn phrase_matches(prompt_lower: &str, phrase: &str) -> bool {
    let phrase_lower = phrase.to_lowercase();
    if prompt_lower.contains(&phrase_lower) {
        return true;
    }

    let prompt_tokens: Vec<&str> = prompt_lower
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect();
    let phrase_tokens: Vec<&str> = phrase_lower
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect();

    if phrase_tokens.is_empty() {
        return false;
    }

    let mut search_index = 0;
    for phrase_token in phrase_tokens {
        if let Some(position) = prompt_tokens[search_index..]
            .iter()
            .position(|prompt_token| prompt_token == &phrase_token)
        {
            search_index += position + 1;
        } else {
            return false;
        }
    }

    true
}
