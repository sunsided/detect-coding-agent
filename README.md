# detect-coding-agent

[![Crates.io](https://img.shields.io/crates/v/detect-coding-agent)](https://crates.io/crates/detect-coding-agent)
[![docs.rs](https://docs.rs/detect-coding-agent/badge.svg)](https://docs.rs/detect-coding-agent)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![License: EUPL-1.2](https://img.shields.io/badge/license-EUPL--1.2-blue.svg)](LICENSE-EUPL)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE-APACHE)

A Rust utility crate that detects whether your application is being invoked by
an AI coding agent such as **Claude Code**, **GitHub Copilot Cloud Agent**,
**OpenAI Codex**, **Cursor**, **Gemini CLI**, **OpenCode**, and many more.

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
detect-coding-agent = "0.2"
```

Then in your code:

```rust
use detect_coding_agent::{detect, is_agent};

fn main() {
    if is_agent() {
        eprintln!("Running inside a coding agent — switching to machine-friendly output.");
    }

    if let Some(agent) = detect() {
        eprintln!("Detected: {} ({}, {} confidence)", agent.name, agent.kind, agent.confidence);
        // e.g. "Detected: Claude Code (agent, high confidence)"
    }
}
```

## Supported agents

Any agent that sets the generic [`AI_AGENT`](#the-ai_agent-convention) variable is
detected, including agents this crate does not know by name. In addition, these
agent-specific signals are recognized (in detection order):

| Agent | Kind | Detection signal | Confidence |
|---|---|---|---|
| [Amp](https://ampcode.com) (Sourcegraph) | Agent | `AMP_CURRENT_THREAD_ID`, `AGENT=amp` | medium |
| Claude Cowork | Agent | `CLAUDE_CODE_IS_COWORK` | medium |
| [Claude Code](https://code.claude.com) (Anthropic) | Agent | `CLAUDE_CODE_CHILD_SESSION=1` / `CLAUDECODE` | high / medium |
| [Kilo Code](https://kilocode.ai) | Agent | `KILO=1` | high |
| [OpenCode](https://opencode.ai) | Agent | `OPENCODE=1`, `OPENCODE_SESSION_ID` | high |
| [Qwen Code](https://github.com/QwenLM/qwen-code) (Alibaba) | Agent | `QWEN_CODE=1` | high |
| [Gemini CLI](https://github.com/google-gemini/gemini-cli) | Agent | `GEMINI_CLI=1` | high |
| [OpenAI Codex](https://github.com/openai/codex) | Agent | `CODEX_THREAD_ID`, `CODEX_SESSION_ID`, `CODEX_CI`, `CODEX_SANDBOX`, `CODEX_SANDBOX_NETWORK_DISABLED` | high |
| [Crush](https://github.com/charmbracelet/crush) (Charm) | Agent | `CRUSH=1`, `AGENT=crush` | high |
| [Pi](https://github.com/badlogic/pi-mono) | Agent | `PI_CODING_AGENT=true` | high |
| [Auggie](https://augmentcode.com) (Augment Code) | Agent | `AUGMENT_AGENT=1` | high |
| GitHub Copilot Cloud Agent | Agent | `COPILOT_AGENT_SESSION_ID` / `COPILOT_AGENT_JOB_ID` / `COPILOT_CLI=1` + `GITHUB_ACTIONS=true` | high |
| [GitHub Copilot CLI](https://github.com/github/copilot-cli) | Agent | `COPILOT_CLI=1`, `COPILOT_AGENT_SESSION_ID` | high |
| GitHub Copilot in VS Code | Agent | `COPILOT_AGENT=1` / `TERM_PROGRAM=vscode` + `GIT_PAGER=cat` | high / low |
| [Cursor](https://cursor.com) Agent (IDE and CLI) | Agent | `CURSOR_AGENT` / `CURSOR_TRACE_ID` + `PAGER=head -n 10000 \| cat` | high / low |
| [Cline](https://cline.bot) | Agent | `CLINE_ACTIVE=true` | high |
| [Roo Code](https://roocode.com) | Agent | `ROO_ACTIVE=true` | high |
| Amazon Q Developer CLI | Agent | `AWS_EXECUTION_ENV` containing `AmazonQ-For-CLI` | high |
| [OpenHands](https://openhands.dev) | Agent | `PS1` / `PROMPT_COMMAND` containing `###PS1JSON###` | medium |
| [Goose](https://github.com/aaif-goose/goose) | Agent | `GOOSE_TERMINAL=1`, `AGENT=goose` / `GOOSE_PROVIDER` | high / low |
| Antigravity (Google) | Agent | `ANTIGRAVITY_AGENT` | medium |
| Junie (JetBrains) | Agent | `JUNIE_DATA`, `JUNIE_SHIM_PATH` | medium |
| Kiro CLI (AWS) | Agent | `KIRO_AGENT_PATH` | medium |
| Trae (ByteDance) | Agent | `TRAE_AI_SHELL_ID` | medium |
| Windsurf Cascade | Agent | `WINDSURF_CASCADE_TERMINAL` | medium |
| Jules (Google) | Agent | `HOME=/home/jules` + `USER=swebot` | low |
| Replit Assistant | Agent | `REPL_ID` + `REPLIT_MODE=assistant` | low |
| Bolt.new Agent | Agent | `SHELL=/bin/jsh` + `npm_config_yes` | low |
| [Zed](https://zed.dev) Agent | Agent | `TERM_PROGRAM=zed` + `PAGER=""` + `GIT_PAGER=cat` | high |
| Warp Agent | Agent | `OZ_RUN_ID` | low |
| Devin | Agent | `EDITOR` containing `devin` / `/opt/.devin` (feature `fs-markers`) / `devin` process | low / high / medium |
| [Aider](https://aider.chat) | Agent | `aider` process (feature `process-detection`) | medium |
| [Factory Droid](https://factory.ai) | Agent | `droid` process (feature `process-detection`) | medium |
| [Octofriend](https://github.com/synthetic-lab/octofriend) | Agent | `octofriend` process (feature `process-detection`) | medium |
| Cursor | Interactive | `CURSOR_TRACE_ID` | low |
| Replit | Interactive | `REPL_ID` | low |
| Bolt.new | Interactive | `SHELL=/bin/jsh` | low |
| Zed | Interactive | `TERM_PROGRAM=zed` | low |
| Kiro | Interactive | `TERM_PROGRAM=kiro` | low |
| Windsurf | Interactive | `CODEIUM_EDITOR_APP_ROOT` | low |
| [Warp Terminal](https://warp.dev) | Hybrid | `TERM_PROGRAM=WarpTerminal` | low |

**Confidence** reflects the evidence behind a signal: *high* signals are injected
deliberately by the agent and verified against its source code or documentation;
*medium* signals are tool-specific but undocumented, or also appear in human
sessions (e.g. `CLAUDECODE` in IDE terminals); *low* signals are ambient or
heuristic. The source of each signal is documented in
[`src/providers.rs`](src/providers.rs).

## The `AI_AGENT` convention

`AI_AGENT` is an emerging cross-tool convention for agents to announce themselves
to the commands they run (see the [agents.md proposal](https://github.com/agentsmd/agents.md/issues/136)).
It is already set by Claude Code, GitHub Copilot in VS Code, Crush, OpenCode, Pi
and OpenHands, and read by tools such as `gh`. Two value formats are understood:

| Value | `id` | `version` | `mode` |
|---|---|---|---|
| `claude-code_2-1-312_agent` | `claude-code` | `2.1.312` | `agent` |
| `devin@1` | `devin` | `1` | – |
| `github_copilot_vscode_agent` | `copilot-vscode` | – | – |
| `some-new-agent` | `unknown` | – | – |

Falsy values (`0`, `false`, `no`, `off`, blank) are ignored. Flag values such as
`AI_AGENT=1` or `AGENT=1` are reported as an `unknown` agent with low confidence,
but only if no agent-specific signal names the agent first.

## API

```rust
// Detect the first matching agent in the current environment
pub fn detect() -> Option<DetectedAgent>

// Detect every matching agent (agents can run inside each other)
pub fn detect_all() -> Vec<DetectedAgent>

// Same, but with an explicit environment map (useful for testing)
pub fn detect_with_env(env: HashMap<String, String>) -> Option<DetectedAgent>
pub fn detect_all_with_env(env: HashMap<String, String>) -> Vec<DetectedAgent>

// Same, plus an explicit process ancestry, nearest ancestor first
pub fn detect_with_env_and_ancestry(env: HashMap<String, String>, ancestry: &[Ancestor]) -> Option<DetectedAgent>
pub fn detect_all_with_env_and_ancestry(env: HashMap<String, String>, ancestry: &[Ancestor]) -> Vec<DetectedAgent>

// Convenience helpers
pub fn is_agent() -> bool       // true for Agent + Hybrid
pub fn is_interactive() -> bool // true for Interactive + Hybrid
pub fn is_hybrid() -> bool      // true for Hybrid only
```

### `DetectedAgent`

```rust
#[non_exhaustive]
pub struct DetectedAgent {
    pub id: &'static str,           // stable identifier, e.g. "claude-code" ("unknown" if unnamed)
    pub name: &'static str,         // human name, e.g. "Claude Code"
    pub kind: AgentKind,            // Agent | Interactive | Hybrid
    pub confidence: Confidence,     // Low | Medium | High
    pub signal: &'static str,       // variable that matched, e.g. "AI_AGENT" (never its value)
    pub version: Option<String>,    // from AI_AGENT, e.g. "2.1.312"
    pub mode: Option<String>,       // from AI_AGENT, e.g. "agent"
    pub ai_agent: Option<String>,   // the validated AI_AGENT value
}
```

### `AgentKind`

```rust
#[non_exhaustive]
pub enum AgentKind {
    Agent,       // Fully automated, no human in the loop
    Interactive, // Human-driven AI-assisted tool
    Hybrid,      // Both capabilities
}
```

## Detection strategy

By default, detection reads **environment variables** only; it never spawns
subprocesses. Two opt-in features let `detect()` and `detect_all()` look further:

- `fs-markers` checks for files that some agents leave behind (currently
  Devin's `/opt/.devin`).
- `process-detection` walks up the process tree looking for known agent
  executables. This is currently implemented for Linux only, by reading `/proc`
  (no subprocesses); on other platforms the ancestry is empty.

`detect_with_env()` and `detect_all_with_env()` never touch the file system;
`detect_with_env_and_ancestry()` takes the ancestry as an argument and works on
every platform. Signals are checked in tiers, and the first match wins:

1. `AI_AGENT` naming an agent.
2. Agent-specific variables. Agents that build on other agents come first, since
   they inherit their parent's variables (Amp sets `CLAUDECODE`, Kilo sets
   `OPENCODE=1`, Qwen Code is a Gemini CLI fork).
3. File markers (feature `fs-markers`, only for `detect()` / `detect_all()`).
4. Process ancestry: the nearest ancestor that is a known agent executable.
   This catches agents that set no environment variables at all (Aider, Devin,
   Factory Droid, Octofriend), and agents whose environment was scrubbed
   (e.g. `env -i`). An ancestor matches by process name, `argv[0]`, or, for
   `node` / `python` / `bun` / `deno`, the script or `-m` module it runs.
5. Ambient signals of IDEs and terminals that also host human sessions.
   The interactive variant of a tool is only reported when its agent variant
   does not match.
6. Anonymous announcements (`AGENT=1`, `AI_AGENT=1`, or `AGENT=<known name>`).

Detection results are a hint, not a security boundary: any process can set these
variables, so never use them for authorization.

## Cargo features

| Feature | Effect |
|---|---|
| `fs-markers` | Also check file-system markers left by agents (`/opt/.devin`). |
| `process-detection` | Also check the process ancestry for known agent executables (Linux only for now, via `/proc`; computed once and cached). |

## Migrating from 0.1

- `DetectedAgent` and `AgentKind` are `#[non_exhaustive]`; `DetectedAgent` gained
  `confidence`, `signal`, `version`, `mode` and `ai_agent`. `AgentKind` is now `Copy`.
- Aider is no longer detected from the environment: `AIDER_API_KEY` is user
  configuration, not a signal Aider injects, and caused false positives. With
  the `process-detection` feature it is detected from the process ancestry.
- `windsurf` is now the interactive IDE terminal; the agent is `windsurf-cascade`.
- The Copilot CLI outside GitHub Actions is reported as `copilot-cli`.
- Zed's agent terminal is detected by `PAGER=""` + `GIT_PAGER=cat`; `PAGER=cat`
  never matched it.
- `OPENCODE_BIN_PATH`, `OPENCODE_SERVER`, `OPENCODE_APP_INFO`, `OPENCODE_MODES`
  and `ANTIGRAVITY_PROJECT_ID` are no longer used; none is injected by the agents.

## Example binary

The crate ships with an example binary that prints detection results:

```sh
cargo run --example detect
```

Sample output when run inside a Claude Code session:

```
✅ Coding agent detected!
   Name       : Claude Code
   ID         : claude-code
   Kind       : agent
   Confidence : high
   Signal     : AI_AGENT
   Version    : 2.1.312
   Mode       : agent
   AI_AGENT   : claude-code_2-1-312_agent

💡 Tip: This is an autonomous agent session — consider offering a
   machine-friendly interface (e.g. via MCP) for better results.
```

## License

Licensed under either of [European Union Public License, Version 1.2](LICENSE-EUPL), [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
