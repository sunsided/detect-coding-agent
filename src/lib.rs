//! # detect-coding-agent
//!
//! Detect whether your application is being invoked by an AI coding agent,
//! such as **Claude Code**, **GitHub Copilot Cloud Agent**, **OpenAI Codex**,
//! **Cursor**, **Aider**, **Gemini CLI**, and [many more](providers).
//!
//! ## Why?
//!
//! When your application knows it is running inside an automated coding agent
//! it can:
//!
//! - Suggest machine-friendly interfaces (e.g. MCP) instead of interactive UIs.
//! - Emit structured output that the agent can parse more reliably.
//! - Skip interactive prompts and use sensible defaults instead.
//! - Provide agent-specific hints or documentation snippets.
//!
//! ## Quick start
//!
//! ```rust
//! use detect_coding_agent::{detect, is_agent};
//!
//! if is_agent() {
//!     println!("Running inside a coding agent — switching to machine-friendly output.");
//! }
//!
//! if let Some(agent) = detect() {
//!     println!("Detected: {} ({})", agent.name, agent.kind);
//! }
//! ```
//!
//! ## Detection strategy
//!
//! Detection is based on **environment variables** only (no subprocesses are
//! spawned, no filesystem access is performed).  The first provider whose
//! signals match the current environment is returned.
//!
//! Each provider has a documented source for its detection heuristic — see
//! [`providers`] for the full list.
//!
//! ## Testing
//!
//! Because detection reads the process environment, tests should supply an
//! explicit environment snapshot via [`detect_with_env`].  Use
//! [`std::collections::HashMap`] to build the environment:
//!
//! ```rust
//! use std::collections::HashMap;
//! use detect_coding_agent::detect_with_env;
//!
//! let mut env = HashMap::new();
//! env.insert("CLAUDECODE".to_string(), "1".to_string());
//!
//! let agent = detect_with_env(env).unwrap();
//! assert_eq!(agent.id, "claude-code");
//! ```

mod env;
pub mod providers;
mod types;

use std::collections::HashMap;

pub use types::{AgentKind, DetectedAgent};

/// Detect the AI coding agent present in the **current process environment**.
///
/// Returns `None` when no known agent is detected.
///
/// # Example
///
/// ```rust
/// if let Some(agent) = detect_coding_agent::detect() {
///     eprintln!("Running inside: {}", agent.name);
/// }
/// ```
pub fn detect() -> Option<DetectedAgent> {
    let env = env::Env::current();
    providers::detect(&env)
}

/// Detect the AI coding agent using a **caller-supplied** environment map.
///
/// This is the testable variant of [`detect`].  Pass a [`HashMap<String, String>`]
/// built from whichever key/value pairs you want to simulate.
///
/// # Example
///
/// ```rust
/// use std::collections::HashMap;
///
/// let mut env = HashMap::new();
/// env.insert("CODEX_THREAD_ID".to_string(), "thread-42".to_string());
///
/// let agent = detect_coding_agent::detect_with_env(env).unwrap();
/// assert_eq!(agent.id, "codex");
/// ```
pub fn detect_with_env(env: HashMap<String, String>) -> Option<DetectedAgent> {
    let env = env::Env::from_map(env);
    providers::detect(&env)
}

/// Returns `true` when the current process is running inside an autonomous
/// coding **agent** or a [`AgentKind::Hybrid`] environment.
///
/// Equivalent to `detect().map(|a| a.is_agent() || a.is_hybrid()).unwrap_or(false)`.
pub fn is_agent() -> bool {
    detect()
        .map(|a| matches!(a.kind, AgentKind::Agent | AgentKind::Hybrid))
        .unwrap_or(false)
}

/// Returns `true` when the current process is running inside an
/// **interactive** AI-assisted coding tool or a [`AgentKind::Hybrid`] environment.
///
/// Equivalent to `detect().map(|a| a.is_interactive() || a.is_hybrid()).unwrap_or(false)`.
pub fn is_interactive() -> bool {
    detect()
        .map(|a| matches!(a.kind, AgentKind::Interactive | AgentKind::Hybrid))
        .unwrap_or(false)
}

/// Returns `true` when the current process is running inside a
/// [`AgentKind::Hybrid`] environment (supports both agent and interactive use).
pub fn is_hybrid() -> bool {
    detect()
        .map(|a| a.kind == AgentKind::Hybrid)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn detect_returns_none_for_empty_env() {
        assert!(detect_with_env(HashMap::new()).is_none());
    }

    #[test]
    fn detect_with_env_finds_claude_code() {
        let agent = detect_with_env(env(&[("CLAUDECODE", "1")])).unwrap();
        assert_eq!(agent.id, "claude-code");
        assert_eq!(agent.name, "Claude Code");
        assert!(agent.is_agent());
        assert!(!agent.is_interactive());
        assert!(!agent.is_hybrid());
    }

    #[test]
    fn detect_with_env_finds_codex() {
        let agent = detect_with_env(env(&[("CODEX_THREAD_ID", "t-1")])).unwrap();
        assert_eq!(agent.id, "codex");
    }

    #[test]
    fn detect_with_env_finds_warp_as_hybrid() {
        let agent = detect_with_env(env(&[("TERM_PROGRAM", "WarpTerminal")])).unwrap();
        assert_eq!(agent.id, "warp");
        assert!(agent.is_hybrid());
        assert!(!agent.is_agent());
        assert!(!agent.is_interactive());
    }

    #[test]
    fn detect_with_env_finds_cursor_interactive() {
        let agent = detect_with_env(env(&[("CURSOR_TRACE_ID", "t-1")])).unwrap();
        assert_eq!(agent.id, "cursor");
        assert!(agent.is_interactive());
    }

    #[test]
    fn agent_kind_display() {
        assert_eq!(AgentKind::Agent.to_string(), "agent");
        assert_eq!(AgentKind::Interactive.to_string(), "interactive");
        assert_eq!(AgentKind::Hybrid.to_string(), "hybrid");
    }

    #[test]
    fn detected_agent_display() {
        let agent = DetectedAgent {
            id: "test",
            name: "Test Agent",
            kind: AgentKind::Agent,
        };
        assert_eq!(agent.to_string(), "Test Agent (agent)");
    }
}
