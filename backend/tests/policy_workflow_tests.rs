use relay_backend::policies::markdown_parser::parse_policy;
use relay_backend::slack::formatter::resolve_slack_text;
use relay_backend::workflows::loader::WorkflowRegistry;
use relay_backend::workflows::matcher::match_workflow;

#[test]
fn parses_policy_markdown() {
    let policy = parse_policy(
        r#"---
name: Example
description: Example policy
---

## Rule One
- id: example-rule
### Match
- review pr
### Examples
- Review PR #123
### Notes
- Example note
"#,
    )
    .expect("policy should parse");

    assert_eq!(policy.rules.len(), 1);
    assert_eq!(policy.rules[0].id, "example-rule");
}

#[test]
fn resolves_slack_mentions_for_display() {
    let resolved = resolve_slack_text("Hello <@U123> see <https://example.com|example>");
    assert!(resolved.contains("@U123"));
    assert!(resolved.contains("example (https://example.com)"));
}

#[test]
fn picks_environment_workflow_when_trigger_matches() {
    let registry = WorkflowRegistry::load(std::path::Path::new("../.workflows"))
        .expect("workflow registry should load");
    let workflow = match_workflow(&registry, "please review pr 42", None).expect("workflow should match");
    assert_eq!(workflow.metadata.id, "pr-review");
}
