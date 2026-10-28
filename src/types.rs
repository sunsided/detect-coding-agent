/// The kind of AI coding environment detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AgentKind {
    /// A fully automated coding agent (no human in the loop during execution).
    ///
    /// Examples: Claude Code, OpenAI Codex, GitHub Copilot Cloud Agent.
    Agent,

    /// An interactive AI-assisted coding tool where a human is still driving.
    ///
    /// Examples: Cursor (interactive terminal), Zed (interactive terminal).
    Interactive,

    /// An environment that can operate in both agent and interactive modes.
    ///
    /// Examples: Warp Terminal.
    Hybrid,
}

impl AgentKind {
    /// Returns `true` if this is an autonomous [`AgentKind::Agent`].
    pub fn is_agent(&self) -> bool {
        matches!(self, Self::Agent)
    }

    /// Returns `true` if this is an [`AgentKind::Interactive`] environment.
    pub fn is_interactive(&self) -> bool {
        matches!(self, Self::Interactive)
    }

    /// Returns `true` if this is a [`AgentKind::Hybrid`] environment.
    pub fn is_hybrid(&self) -> bool {
        matches!(self, Self::Hybrid)
    }
}

impl std::fmt::Display for AgentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Agent => write!(f, "agent"),
            Self::Interactive => write!(f, "interactive"),
            Self::Hybrid => write!(f, "hybrid"),
        }
    }
}

/// How strongly the matched signal indicates the detected agent.
///
/// Variants are ordered from weakest to strongest, so
/// `Confidence::Low < Confidence::High`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Confidence {
    /// A weak or ambient signal that other tools may also set
    /// (e.g. `TERM_PROGRAM`, `REPL_ID`), or a heuristic without a primary source.
    Low,

    /// A tool-specific signal that is not documented by the vendor, or that
    /// is also present in some non-agent contexts (e.g. `CLAUDECODE` in IDE terminals).
    Medium,

    /// A signal the agent deliberately injects into the commands it runs,
    /// verified against its source code or documentation, or an explicit `AI_AGENT` value.
    High,
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}

/// Information about a detected AI coding agent environment.
///
/// Detection results are a hint, not a security boundary: any process can set
/// these environment variables, so never use them for authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DetectedAgent {
    /// A stable, unique identifier for the agent (e.g. `"claude-code"`, `"codex"`).
    ///
    /// `"unknown"` when an agent announced itself via `AI_AGENT` or `AGENT`
    /// but is not known to this crate; see [`DetectedAgent::ai_agent`].
    pub id: &'static str,

    /// The human-readable name of the agent (e.g. `"Claude Code"`, `"OpenAI Codex"`).
    pub name: &'static str,

    /// The kind of AI coding environment.
    pub kind: AgentKind,

    /// How strongly [`DetectedAgent::signal`] indicates this agent.
    pub confidence: Confidence,

    /// The name of the environment variable (or marker) that triggered the detection,
    /// e.g. `"AI_AGENT"` or `"CODEX_THREAD_ID"`.
    ///
    /// Only the name is reported; values are never captured because some of
    /// them may contain secrets.
    pub signal: &'static str,

    /// The agent's version, when announced via `AI_AGENT`
    /// (e.g. `"2.1.312"` for `AI_AGENT=claude-code_2-1-312_agent`).
    pub version: Option<String>,

    /// The agent's mode, when announced via `AI_AGENT`
    /// (e.g. `"agent"` for `AI_AGENT=claude-code_2-1-312_agent`).
    pub mode: Option<String>,

    /// The validated, verbatim `AI_AGENT` value, if one was set.
    pub ai_agent: Option<String>,
}

impl DetectedAgent {
    /// Returns `true` if the detected environment is an autonomous agent.
    pub fn is_agent(&self) -> bool {
        self.kind.is_agent()
    }

    /// Returns `true` if the detected environment is interactive (human-driven).
    pub fn is_interactive(&self) -> bool {
        self.kind.is_interactive()
    }

    /// Returns `true` if the detected environment is hybrid (agent + interactive).
    pub fn is_hybrid(&self) -> bool {
        self.kind.is_hybrid()
    }

    /// Returns `true` if the agent announced itself but is not known to this crate.
    pub fn is_unknown(&self) -> bool {
        self.id == UNKNOWN_ID
    }
}

impl std::fmt::Display for DetectedAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.kind)
    }
}

/// Identifier used for agents that announce themselves but are not known to this crate.
pub(crate) const UNKNOWN_ID: &str = "unknown";
