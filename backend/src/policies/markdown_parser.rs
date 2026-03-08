use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PolicyDocumentMeta {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub id: String,
    pub title: String,
    pub match_terms: Vec<String>,
    pub examples: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PolicyDocument {
    pub meta: PolicyDocumentMeta,
    pub rules: Vec<PolicyRule>,
}

pub fn load_policy(path: &Path) -> Result<PolicyDocument> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    parse_policy(&content).with_context(|| format!("failed to parse {}", path.display()))
}

pub fn parse_policy(content: &str) -> Result<PolicyDocument> {
    let mut parts = content.splitn(3, "---");
    let leading = parts.next().unwrap_or_default();
    if !leading.trim().is_empty() {
        return Err(anyhow!("policy must start with YAML frontmatter"));
    }

    let frontmatter = parts
        .next()
        .ok_or_else(|| anyhow!("missing YAML frontmatter section"))?;
    let body = parts
        .next()
        .ok_or_else(|| anyhow!("missing markdown body after frontmatter"))?;

    let meta: PolicyDocumentMeta = serde_yaml::from_str(frontmatter)?;
    let mut rules = Vec::new();
    let mut current_title: Option<String> = None;
    let mut lines = Vec::new();

    for line in body.lines() {
        if let Some(title) = line.strip_prefix("## ") {
            if let Some(title) = current_title.take() {
                rules.push(parse_rule_block(&title, &lines.join("\n"))?);
                lines.clear();
            }
            current_title = Some(title.trim().to_string());
        } else if current_title.is_some() {
            lines.push(line.to_string());
        }
    }

    if let Some(title) = current_title.take() {
        rules.push(parse_rule_block(&title, &lines.join("\n"))?);
    }

    if rules.is_empty() {
        return Err(anyhow!(
            "policy file must contain at least one rule section"
        ));
    }

    Ok(PolicyDocument { meta, rules })
}

fn parse_rule_block(title: &str, body: &str) -> Result<PolicyRule> {
    let mut id = None;
    let mut match_terms = Vec::new();
    let mut examples = Vec::new();
    let mut notes = Vec::new();

    enum Section {
        None,
        Match,
        Examples,
        Notes,
    }

    let mut section = Section::None;

    for raw_line in body.lines() {
        let line = raw_line.trim();
        if let Some(value) = line.strip_prefix("- id:") {
            id = Some(value.trim().to_string());
            continue;
        }

        if line.eq_ignore_ascii_case("### Match") {
            section = Section::Match;
            continue;
        }
        if line.eq_ignore_ascii_case("### Examples") {
            section = Section::Examples;
            continue;
        }
        if line.eq_ignore_ascii_case("### Notes") {
            section = Section::Notes;
            continue;
        }

        if let Some(value) = line.strip_prefix("- ") {
            match section {
                Section::Match => match_terms.push(value.trim().to_string()),
                Section::Examples => examples.push(value.trim().to_string()),
                Section::Notes => notes.push(value.trim().to_string()),
                Section::None => {}
            }
        }
    }

    let id = id.ok_or_else(|| anyhow!("rule '{title}' missing '- id:' line"))?;
    if match_terms.is_empty() {
        return Err(anyhow!("rule '{title}' must define match terms"));
    }
    if examples.is_empty() {
        return Err(anyhow!("rule '{title}' must define examples"));
    }

    Ok(PolicyRule {
        id,
        title: title.to_string(),
        match_terms,
        examples,
        notes: (!notes.is_empty()).then(|| notes.join("\n")),
    })
}
