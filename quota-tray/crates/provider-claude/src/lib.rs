//! Claude Code credentials + Anthropic OAuth usage fetch.

pub fn provider_id_str() -> &'static str {
    "claude"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_id_str_is_claude() {
        assert_eq!(provider_id_str(), "claude");
    }
}
