//! Claude Code hook payload parsing and JSONL event storage.

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct HookPayload {
    pub session_id: Option<String>,
    pub hook_event_name: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_use_id: Option<String>,
    pub tool_response: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionEvent {
    pub session_id: String,
    pub timestamp: String,
    pub kind: String,
    pub data: serde_json::Value,
}

/// Parse a Claude Code hook payload from JSON string.
/// Returns an InteractionEvent if the payload is a Playwright tool use.
pub fn parse_hook_payload(input: &str) -> Option<InteractionEvent> {
    let payload: HookPayload = serde_json::from_str(input).ok()?;

    let tool_name = payload.tool_name.as_deref()?;
    if !tool_name.starts_with("mcp__playwright__") {
        return None;
    }

    let kind = match tool_name.strip_prefix("mcp__playwright__") {
        Some("browser_navigate") => "navigate",
        Some("browser_click") => "click",
        Some("browser_fill") => "fill",
        Some("browser_screenshot") | Some("browser_take_screenshot") => "screenshot",
        Some("browser_snapshot") => "snapshot",
        _ => "other",
    };

    let timestamp = chrono::Utc::now().to_rfc3339();
    let session_id = payload.session_id.unwrap_or_default();

    Some(InteractionEvent {
        session_id,
        timestamp,
        kind: kind.to_string(),
        data: serde_json::json!({
            "tool": tool_name,
            "input": payload.tool_input,
            "hook_event": payload.hook_event_name,
        }),
    })
}

/// Append an event to a JSONL file.
pub fn append_event(log_path: &Path, event: &InteractionEvent) -> Result<(), String> {
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir failed: {}", e))?;
    }

    let json = serde_json::to_string(event).map_err(|e| format!("serialize: {}", e))?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(|e| format!("open: {}", e))?;

    writeln!(file, "{}", json).map_err(|e| format!("write: {}", e))?;
    Ok(())
}
