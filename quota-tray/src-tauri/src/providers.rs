use provider_claude::ClaudeProvider;
use provider_copilot::CopilotProvider;
use provider_cursor::CursorProvider;
use quota_tray_core::{AppConfig, Provider, ProviderId};

/// Build the poller provider list from config (enabled + Claude headline).
pub fn from_config(config: &AppConfig) -> Vec<Box<dyn Provider>> {
    let mut out: Vec<Box<dyn Provider>> = Vec::new();
    if config.enabled.contains(&ProviderId::Claude) {
        out.push(Box::new(ClaudeProvider {
            headline: config.claude_headline.clone(),
            ..Default::default()
        }));
    }
    if config.enabled.contains(&ProviderId::Cursor) {
        out.push(Box::new(CursorProvider::default()));
    }
    if config.enabled.contains(&ProviderId::Copilot) {
        out.push(Box::new(CopilotProvider::default()));
    }
    // OpenAI: register when that crate lands.
    out
}
