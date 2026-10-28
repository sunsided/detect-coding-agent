//! Parsing of the generic `AI_AGENT` environment variable.
//!
//! `AI_AGENT` is an emerging cross-tool convention for agents to announce
//! themselves to the commands they run. Two value dialects are in use:
//!
//! - `name[@version]`, promoted by Vercel (e.g. `devin@1`, `custom-agent@2.0`)
//! - `name_version_mode`, used by Claude Code (e.g. `claude-code_2-1-312_agent`,
//!   where dashes in the version stand for dots)
//!
//! Some agents use an underscored name without version (e.g. VS Code's
//! `github_copilot_vscode_agent`), so the whole value is always tried as a known
//! name before it is split.
//!
//! See <https://github.com/agentsmd/agents.md/issues/136> for the
//! standardization proposal and <https://github.com/vercel/vercel/tree/main/packages/detect-agent>
//! for the `name@version` dialect.

use crate::env::{Env, is_truthy};

/// The environment variable carrying the agent announcement.
pub const AI_AGENT: &str = "AI_AGENT";

/// Maximum accepted length of an `AI_AGENT` value.
const MAX_LEN: usize = 64;

/// What the `AI_AGENT` variable says about the environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Announcement {
    /// The agent named itself.
    Named(AiAgent),

    /// `AI_AGENT` is set to a truthy value that does not name an agent
    /// (e.g. `1`, `true`), or to a value that fails validation.
    Flag,
}

/// A parsed, validated `AI_AGENT` value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiAgent {
    /// The verbatim (validated) value.
    pub raw: String,

    /// Lowercased candidate names to look up, most specific first:
    /// the whole value, then the parsed name.
    pub candidates: Vec<String>,

    /// The version, if the value carried one.
    pub version: Option<String>,

    /// The mode, if the value carried one (`name_version_mode` dialect only).
    pub mode: Option<String>,
}

/// Reads and parses `AI_AGENT` from `env`.
///
/// Returns `None` when the variable is unset or falsy (`0`, `false`, `no`, `off`, blank).
pub fn read(env: &Env) -> Option<Announcement> {
    parse(env.get(AI_AGENT)?)
}

/// Parses an `AI_AGENT` value. See [`read`].
pub fn parse(value: &str) -> Option<Announcement> {
    let value = value.trim();
    if !is_truthy(value) {
        return None;
    }

    if !is_valid(value) || is_flag(value) {
        return Some(Announcement::Flag);
    }

    let whole = value.to_ascii_lowercase();
    let (name, version, mode) = if let Some((name, version)) = whole.split_once('@') {
        (name.to_owned(), non_empty(version), None)
    } else {
        split_underscored(&whole)
    };

    if name.is_empty() {
        return Some(Announcement::Flag);
    }

    let mut candidates = vec![whole];
    if candidates[0] != name {
        candidates.push(name);
    }

    Some(Announcement::Named(AiAgent {
        raw: value.to_owned(),
        candidates,
        version,
        mode,
    }))
}

/// Accepts the header-safe character set used by `gh` plus `.` and `@`
/// for the `name@version` dialect.
fn is_valid(value: &str) -> bool {
    value.len() <= MAX_LEN
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'@'))
}

/// Values that only signal "some agent is present" without naming it.
fn is_flag(value: &str) -> bool {
    value.bytes().all(|b| b.is_ascii_digit())
        || ["true", "yes", "on"]
            .iter()
            .any(|flag| value.eq_ignore_ascii_case(flag))
}

/// Splits `name_version_mode`: the name is everything before the first
/// underscore-separated part that starts with a digit.
fn split_underscored(value: &str) -> (String, Option<String>, Option<String>) {
    let parts: Vec<&str> = value.split('_').collect();
    let Some(index) = parts
        .iter()
        .position(|part| part.starts_with(|c: char| c.is_ascii_digit()))
    else {
        return (value.to_owned(), None, None);
    };

    let name = parts[..index].join("_");
    let version = Some(parts[index].replace('-', "."));
    let mode = non_empty(&parts[index + 1..].join("_"));
    (name, version, mode)
}

fn non_empty(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(value: &str) -> AiAgent {
        match parse(value) {
            Some(Announcement::Named(agent)) => agent,
            other => panic!("expected named announcement for {value:?}, got {other:?}"),
        }
    }

    #[test]
    fn parses_claude_code_dialect() {
        let agent = named("claude-code_2-1-312_agent");
        assert_eq!(agent.raw, "claude-code_2-1-312_agent");
        assert_eq!(
            agent.candidates,
            ["claude-code_2-1-312_agent", "claude-code"]
        );
        assert_eq!(agent.version.as_deref(), Some("2.1.312"));
        assert_eq!(agent.mode.as_deref(), Some("agent"));
    }

    #[test]
    fn parses_at_version_dialect() {
        let agent = named("Devin@1");
        assert_eq!(agent.candidates, ["devin@1", "devin"]);
        assert_eq!(agent.version.as_deref(), Some("1"));
        assert_eq!(agent.mode, None);
    }

    #[test]
    fn keeps_underscored_names_without_version_whole() {
        let agent = named("github_copilot_vscode_agent");
        assert_eq!(agent.candidates, ["github_copilot_vscode_agent"]);
        assert_eq!(agent.version, None);
        assert_eq!(agent.mode, None);
    }

    #[test]
    fn plain_name() {
        let agent = named("crush");
        assert_eq!(agent.candidates, ["crush"]);
        assert_eq!(agent.version, None);
    }

    #[test]
    fn ignores_falsy_values() {
        for value in ["", "  ", "0", "false", "No", "off"] {
            assert_eq!(parse(value), None, "{value:?}");
        }
    }

    #[test]
    fn flags_and_invalid_values_are_anonymous() {
        let too_long = "a".repeat(MAX_LEN + 1);
        for value in [
            "1",
            "true",
            "YES",
            "has space",
            "semi;colon",
            "@1",
            &too_long,
        ] {
            assert_eq!(parse(value), Some(Announcement::Flag), "{value:?}");
        }
    }
}
