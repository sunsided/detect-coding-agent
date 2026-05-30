/// The kind of AI coding environment detected.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AgentKind {
    /// A fully automated coding agent (no human in the loop during execution).
    ///
    /// Examples: Claude Code, OpenAI Codex, GitHub Copilot Cloud Agent.
    Agent,

    /// An interactive AI-assisted coding tool where a human is still driving.
    ///
    /// Examples: Cursor (interactive mode), Zed (interactive mode).
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

/// Information about a detected AI coding agent environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedAgent {
    /// A stable, unique identifier for the agent (e.g. `"claude-code"`, `"codex"`).
    pub id: &'static str,

    /// The human-readable name of the agent (e.g. `"Claude Code"`, `"OpenAI Codex"`).
    pub name: &'static str,

    /// The kind of AI coding environment.
    pub kind: AgentKind,
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
}

impl std::fmt::Display for DetectedAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.kind)
    }
}
