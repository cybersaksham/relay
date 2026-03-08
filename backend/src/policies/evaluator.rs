use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;

use super::markdown_parser::{load_policy, PolicyDocument, PolicyRule};

#[derive(Debug, Clone)]
pub enum PolicyDecision {
    Allowed,
    NonMasterDenied(PolicyRule),
    CriticalDenied(PolicyRule),
}

#[derive(Debug, Clone)]
pub struct PolicyEngine {
    masters: HashSet<String>,
    non_master: PolicyDocument,
    critical_deny: PolicyDocument,
}

impl PolicyEngine {
    pub fn load(policies_dir: &Path, masters: HashSet<String>) -> Result<Self> {
        let non_master = load_policy(&policies_dir.join("non-master.md"))?;
        let critical_deny = load_policy(&policies_dir.join("critical-deny.md"))?;
        Ok(Self {
            masters,
            non_master,
            critical_deny,
        })
    }

    pub fn is_master(&self, slack_user_id: &str) -> bool {
        self.masters.contains(slack_user_id)
    }

    pub fn evaluate(&self, slack_user_id: &str, request: &str) -> PolicyDecision {
        if let Some(rule) = self.match_rule(&self.critical_deny, request) {
            return PolicyDecision::CriticalDenied(rule);
        }

        if self.is_master(slack_user_id) {
            return PolicyDecision::Allowed;
        }

        if let Some(rule) = self.match_rule(&self.non_master, request) {
            let _ = rule;
            PolicyDecision::Allowed
        } else {
            PolicyDecision::NonMasterDenied(PolicyRule {
                id: "non-master-denied".to_string(),
                title: self.non_master.meta.name.clone(),
                match_terms: Vec::new(),
                examples: Vec::new(),
                notes: self.non_master.meta.description.clone(),
            })
        }
    }

    fn match_rule(&self, document: &PolicyDocument, request: &str) -> Option<PolicyRule> {
        let request = request.to_lowercase();
        document.rules.iter().find_map(|rule| {
            rule.match_terms
                .iter()
                .any(|pattern| request.contains(&pattern.to_lowercase()))
                .then(|| rule.clone())
        })
    }
}
