# detect-coding-agent

[![Crates.io](https://img.shields.io/crates/v/detect-coding-agent)](https://crates.io/crates/detect-coding-agent)
[![docs.rs](https://docs.rs/detect-coding-agent/badge.svg)](https://docs.rs/detect-coding-agent)
[![License](https://img.shields.io/crates/l/detect-coding-agent)](LICENSE-MIT)

A Rust utility crate that detects whether your application is being invoked by
an AI coding agent such as **Claude Code**, **GitHub Copilot Cloud Agent**,
**OpenAI Codex**, **Cursor**, **Aider**, **Gemini CLI**, and many more.

## Why?

When your application knows it is running inside an automated coding agent it can:

- Suggest machine-friendly interfaces (e.g. MCP) instead of interactive UIs.
- Emit structured output that the agent can parse more reliably.
- Skip interactive prompts and use sensible defaults instead.
- Provide agent-specific hints or documentation snippets.

## Quick start

Add to your `Cargo.toml`:

```toml
[dependencies]
detect-coding-agent = "0.1"
```

Then in your code:

```rust
use detect_coding_agent::{detect, is_agent};

fn main() {
    if is_agent() {
        eprintln!("Running inside a coding agent — switching to machine-friendly output.");
    }

    if let Some(agent) = detect() {
        eprintln!("Detected: {} ({})", agent.name, agent.kind);
        // e.g. "Detected: Claude Code (agent)"
    }
}
```

## Supported agents

| Agent | Kind | Detection signal |
|---|---|---|
| [OpenCode](https://opencode.ai) | Agent | `OPENCODE`, `OPENCODE_BIN_PATH`, `OPENCODE_SERVER`, `OPENCODE_APP_INFO`, `OPENCODE_MODES` |
| Jules (Google) | Agent | `HOME=/home/jules` + `USER=swebot` |
| [Claude Code](https://docs.anthropic.com/en/docs/claude-code) (Anthropic) | Agent | `CLAUDECODE` |
| [Gemini CLI](https://github.com/google-gemini/gemini-cli) | Agent | `GEMINI_CLI=1` |
| [OpenAI Codex](https://github.com/openai/codex) | Agent | `CODEX_THREAD_ID` |
| [Aider](https://aider.chat) | Agent | `AIDER_API_KEY` |
| [Windsurf](https://windsurf.com) (Codeium) | Agent | `CODEIUM_EDITOR_APP_ROOT` |
| Antigravity | Agent | `ANTIGRAVITY_AGENT`, `ANTIGRAVITY_PROJECT_ID` |
| [Crush](https://github.com/charmbracelet/crush) (Charm) | Agent | `CRUSH=1`, `AGENT=crush`, `AI_AGENT=crush` |
| [Amp](https://ampcode.com) (Sourcegraph) | Agent | `AMP_CURRENT_THREAD_ID`, `AGENT=amp` |
| [Auggie](https://augmentcode.com) (Augment Code) | Agent | `AUGMENT_AGENT=1` |
| Qwen Code (Alibaba) | Agent | `QWEN_CODE=1` |
| GitHub Copilot Cloud Agent | Agent | `COPILOT_AGENT_SESSION_ID` + `GITHUB_ACTIONS=true` |
| Cursor Agent | Agent | `CURSOR_TRACE_ID` + `PAGER=head -n 10000 \| cat` |
| Cursor | Interactive | `CURSOR_TRACE_ID` |
| Replit Assistant | Agent | `REPL_ID` + `REPLIT_MODE=assistant` |
| Replit | Interactive | `REPL_ID` |
| Bolt.new Agent | Agent | `SHELL=/bin/jsh` + `npm_config_yes` |
| Bolt.new | Interactive | `SHELL=/bin/jsh` |
| Zed Agent | Agent | `TERM_PROGRAM=zed` + `PAGER=cat` |
| Zed | Interactive | `TERM_PROGRAM=zed` |
| GitHub Copilot in VS Code | Agent | `TERM_PROGRAM=vscode` + `GIT_PAGER=cat` |
| [Warp Terminal](https://warp.dev) | Hybrid | `TERM_PROGRAM=WarpTerminal` |

## API

```rust
// Detect the first matching agent in the current environment
pub fn detect() -> Option<DetectedAgent>

// Same, but with an explicit environment map (useful for testing)
pub fn detect_with_env(env: HashMap<String, String>) -> Option<DetectedAgent>

// Convenience helpers
pub fn is_agent() -> bool       // true for Agent + Hybrid
pub fn is_interactive() -> bool // true for Interactive + Hybrid
pub fn is_hybrid() -> bool      // true for Hybrid only
```

### `DetectedAgent`

```rust
pub struct DetectedAgent {
    pub id: &'static str,     // stable identifier, e.g. "claude-code"
    pub name: &'static str,   // human name, e.g. "Claude Code"
    pub kind: AgentKind,      // Agent | Interactive | Hybrid
}
```

### `AgentKind`

```rust
pub enum AgentKind {
    Agent,       // Fully automated, no human in the loop
    Interactive, // Human-driven AI-assisted tool
    Hybrid,      // Both capabilities
}
```

## Detection strategy

Detection is based exclusively on **environment variables** — no subprocesses
are spawned and no filesystem access is performed.  The first provider whose
signals match the current environment is returned.

The ordering of checks matters for providers that share environment signals
(e.g. agent mode vs. interactive mode of the same tool).  More specific checks
(e.g. Cursor agent, identified by both `CURSOR_TRACE_ID` *and* a specific
`PAGER` value) are evaluated before the less specific interactive variant
(identified by `CURSOR_TRACE_ID` alone).

## Example binary

The crate ships with an example binary that prints detection results:

```sh
cargo run --example detect
```

Sample output when run inside a GitHub Copilot Cloud Agent session:

```
✅ Coding agent detected!
   Name : GitHub Copilot Cloud Agent
   ID   : copilot-cloud-agent
   Kind : agent

💡 Tip: This is an autonomous agent session — consider offering a
   machine-friendly interface (e.g. via MCP) for better results.
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.