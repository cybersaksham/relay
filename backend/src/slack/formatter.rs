use regex::Regex;
use serde_json::json;

pub fn resolve_slack_text(text: &str) -> String {
    let user = Regex::new(r"<@([A-Z0-9]+)>").expect("valid regex");
    let channel = Regex::new(r"<#([A-Z0-9]+)\|([^>]+)>").expect("valid regex");
    let group = Regex::new(r"<!subteam\^[^|]+\|([^>]+)>").expect("valid regex");
    let link = Regex::new(r"<(https?://[^>|]+)\|([^>]+)>").expect("valid regex");

    let text = user.replace_all(text, "@$1").to_string();
    let text = channel.replace_all(&text, "#$2").to_string();
    let text = group.replace_all(&text, "@$1").to_string();
    link.replace_all(&text, "$2 ($1)").to_string()
}

pub fn resolved_payload_json(text: &str) -> String {
    json!({
        "text": resolve_slack_text(text),
    })
    .to_string()
}
