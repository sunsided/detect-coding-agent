//! # detect-coding-agent
//!
//! Detect whether your application is being invoked by an AI coding agent,
//! such as **Claude Code**, **GitHub Copilot Cloud Agent**, **OpenAI Codex**,
//! **Cursor**, **Gemini CLI**, **OpenCode**, and [many more](providers).
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
//! spawned, no filesystem access is performed unless the `fs-markers` feature
//! is enabled).  Signals are checked in tiers:
//!
//! 1. The generic **`AI_AGENT`** variable, an emerging cross-tool convention
//!    (`AI_AGENT=claude-code_2-1-312_agent`, `AI_AGENT=devin@1`).  Its version
//!    and mode are reported in [`DetectedAgent::version`] and [`DetectedAgent::mode`].
//! 2. **Agent-specific** variables that agents inject into the commands they run.
//! 3. **File markers** (feature `fs-markers`, [`detect`] / [`detect_all`] only).
//! 4. **Ambient** signals of IDEs and terminals that also host human sessions.
//! 5. **Anonymous** announcements (`AGENT=1`, `AI_AGENT=1`), reported as `"unknown"`.
//!
//! [`detect`] returns the first match; [`detect_all`] returns every match,
//! which is useful when agents run inside each other.  Each result carries a
//! [`Confidence`] and the name of the [`signal`](DetectedAgent::signal) that matched.
//!
//! See [`providers`] for the full list of agents and the source of each signal.
//!
//! Detection results are a hint, not a security boundary: any process can set
//! these variables, so never use them for authorization.
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

#![forbid(unsafe_code)]

mod ai_agent;
mod env;
pub mod providers;
mod types;

use std::collections::HashMap;

pub use types::{AgentKind, Confidence, DetectedAgent};

/// Whether file-system markers are checked for the current process.
const FILE_MARKERS: bool = cfg!(feature = "fs-markers");

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
    providers::detect(&env::Env::current(), FILE_MARKERS)
}

/// Detect **all** AI coding agents present in the current process environment,
/// in detection order and at most once per agent id.
///
/// Agents that run inside other agents (or inside an AI-enabled terminal)
/// inherit their parent's variables, so more than one agent can match.
/// The first element is what [`detect`] returns.
pub fn detect_all() -> Vec<DetectedAgent> {
    providers::detect_all(&env::Env::current(), FILE_MARKERS)
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
///
/// File markers (feature `fs-markers`) are never checked, so the result
/// depends on `env` alone.
pub fn detect_with_env(env: HashMap<String, String>) -> Option<DetectedAgent> {
    providers::detect(&env::Env::from_map(env), false)
}

/// Detect all AI coding agents using a **caller-supplied** environment map.
///
/// This is the testable variant of [`detect_all`].
pub fn detect_all_with_env(env: HashMap<String, String>) -> Vec<DetectedAgent> {
    providers::detect_all(&env::Env::from_map(env), false)
}

/// Returns `true` when the current process is running inside an autonomous
/// coding **agent** or a [`AgentKind::Hybrid`] environment.
///
/// Equivalent to `detect().is_some_and(|a| a.is_agent() || a.is_hybrid())`.
pub fn is_agent() -> bool {
    detect().is_some_and(|a| a.is_agent() || a.is_hybrid())
}

/// Returns `true` when the current process is running inside an
/// **interactive** AI-assisted coding tool or a [`AgentKind::Hybrid`] environment.
///
/// Equivalent to `detect().is_some_and(|a| a.is_interactive() || a.is_hybrid())`.
pub fn is_interactive() -> bool {
    detect().is_some_and(|a| a.is_interactive() || a.is_hybrid())
}

/// Returns `true` when the current process is running inside a
/// [`AgentKind::Hybrid`] environment (supports both agent and interactive use).
pub fn is_hybrid() -> bool {
    detect().is_some_and(|a| a.is_hybrid())
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
        let agent = detect_with_env(env(&[("CLAUDE_CODE_CHILD_SESSION", "1")])).unwrap();
        assert_eq!(agent.id, "claude-code");
        assert_eq!(agent.name, "Claude Code");
        assert_eq!(agent.confidence, Confidence::High);
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
    fn detect_all_with_env_lists_every_match() {
        let all = detect_all_with_env(env(&[
            ("AI_AGENT", "claude-code_2-1-312_agent"),
            ("TERM_PROGRAM", "WarpTerminal"),
        ]));
        let ids: Vec<_> = all.iter().map(|a| a.id).collect();
        assert_eq!(ids, ["claude-code", "warp"]);
        assert_eq!(all[0].version.as_deref(), Some("2.1.312"));
    }

    #[test]
    fn confidence_ordering_and_display() {
        assert!(Confidence::Low < Confidence::Medium);
        assert!(Confidence::Medium < Confidence::High);
        assert_eq!(Confidence::High.to_string(), "high");
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
            confidence: Confidence::High,
            signal: "TEST",
            version: None,
            mode: None,
            ai_agent: None,
        };
        assert_eq!(agent.to_string(), "Test Agent (agent)");
    }
}
