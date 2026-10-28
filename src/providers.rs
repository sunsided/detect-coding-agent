//! Known AI coding agents and the signals used to detect them.
//!
//! Detection runs in tiers; within a tier, the first matching provider wins:
//!
//! 1. **`AI_AGENT`** naming a known agent (or an unknown one, reported as `"unknown"`).
//! 2. **Agent-specific signals** that agents inject into the commands they run.
//!    Derivatives come before the tools they build on, because they inherit
//!    their parent's variables (e.g. Amp before Claude Code, Kilo before OpenCode,
//!    Qwen Code before Gemini CLI).
//! 3. **Optional file markers** (cargo feature `fs-markers`, current process only).
//! 4. **Process ancestry** (cargo feature `process-detection` for the current
//!    process, Linux only): the nearest ancestor that is a known agent executable.
//! 5. **Ambient signals** of IDEs and terminals that also host human sessions
//!    (e.g. `TERM_PROGRAM`, `REPL_ID`).
//! 6. **Anonymous announcements**: a known name in the legacy `AGENT` variable,
//!    or `AGENT` / `AI_AGENT` set to a flag value such as `1`.
//!
//! Every matcher documents where its signal comes from. Signals without a
//! primary (vendor) source are reported with [`Confidence::Medium`] or
//! [`Confidence::Low`].
//!
//! | Agent | Kind | Signal(s) |
//! |---|---|---|
//! | Amp | agent | `AMP_CURRENT_THREAD_ID`, `AGENT=amp` |
//! | Claude Cowork | agent | `CLAUDE_CODE_IS_COWORK` |
//! | Claude Code | agent | `CLAUDE_CODE_CHILD_SESSION=1`, `CLAUDECODE` |
//! | Kilo Code | agent | `KILO=1` |
//! | OpenCode | agent | `OPENCODE=1`, `OPENCODE_SESSION_ID` |
//! | Qwen Code | agent | `QWEN_CODE=1` |
//! | Gemini CLI | agent | `GEMINI_CLI=1` |
//! | OpenAI Codex | agent | `CODEX_THREAD_ID`, `CODEX_SESSION_ID`, `CODEX_CI`, `CODEX_SANDBOX`, `CODEX_SANDBOX_NETWORK_DISABLED` |
//! | Crush | agent | `CRUSH=1`, `AGENT=crush` |
//! | Pi | agent | `PI_CODING_AGENT=true` |
//! | Auggie | agent | `AUGMENT_AGENT=1` |
//! | GitHub Copilot Cloud Agent | agent | `COPILOT_AGENT_SESSION_ID` / `COPILOT_AGENT_JOB_ID` / `COPILOT_CLI=1` + `GITHUB_ACTIONS=true` |
//! | GitHub Copilot CLI | agent | `COPILOT_CLI=1`, `COPILOT_AGENT_SESSION_ID` |
//! | GitHub Copilot in VS Code | agent | `COPILOT_AGENT=1`; `TERM_PROGRAM=vscode` + `GIT_PAGER=cat` |
//! | Cursor Agent | agent | `CURSOR_AGENT`; `CURSOR_TRACE_ID` + `PAGER=head -n 10000 \| cat` |
//! | Cline | agent | `CLINE_ACTIVE=true` |
//! | Roo Code | agent | `ROO_ACTIVE=true` |
//! | Amazon Q Developer CLI | agent | `AWS_EXECUTION_ENV` containing `AmazonQ-For-CLI` |
//! | OpenHands | agent | `PS1` / `PROMPT_COMMAND` containing `###PS1JSON###` |
//! | Goose | agent | `GOOSE_TERMINAL=1`, `AGENT=goose`; `GOOSE_PROVIDER` (ambient) |
//! | Antigravity | agent | `ANTIGRAVITY_AGENT` |
//! | Junie | agent | `JUNIE_DATA`, `JUNIE_SHIM_PATH` |
//! | Kiro CLI | agent | `KIRO_AGENT_PATH` |
//! | Trae | agent | `TRAE_AI_SHELL_ID` |
//! | Windsurf Cascade | agent | `WINDSURF_CASCADE_TERMINAL` |
//! | Jules | agent | `HOME=/home/jules` + `USER=swebot` |
//! | Replit Assistant | agent | `REPL_ID` + `REPLIT_MODE=assistant` |
//! | Bolt.new Agent | agent | `SHELL=/bin/jsh` + `npm_config_yes` |
//! | Zed Agent | agent | `TERM_PROGRAM=zed` + `PAGER=""` + `GIT_PAGER=cat` |
//! | Warp Agent | agent | `OZ_RUN_ID` |
//! | Devin | agent | `EDITOR` containing `devin`; file `/opt/.devin` (`fs-markers`); process `devin` |
//! | Aider | agent | process `aider` |
//! | Factory Droid | agent | process `droid` |
//! | Octofriend | agent | process `octofriend` |
//! | Cursor | interactive | `CURSOR_TRACE_ID` |
//! | Replit | interactive | `REPL_ID` |
//! | Bolt.new | interactive | `SHELL=/bin/jsh` |
//! | Zed | interactive | `TERM_PROGRAM=zed` |
//! | Kiro | interactive | `TERM_PROGRAM=kiro` |
//! | Windsurf | interactive | `CODEIUM_EDITOR_APP_ROOT` |
//! | Warp Terminal | hybrid | `TERM_PROGRAM=WarpTerminal` |
//!
//! Claude Code, OpenAI Codex, Gemini CLI, Qwen Code, Crush and OpenCode are
//! also recognized as ancestor processes, for commands run with a scrubbed environment.

use crate::ai_agent::{self, AI_AGENT, AiAgent, Announcement};
use crate::env::Env;
use crate::process::Ancestor;
use crate::types::{AgentKind, Confidence, DetectedAgent, UNKNOWN_ID};

/// A provider definition: a static description of an AI coding agent and
/// the function used to detect its presence in the environment.
pub(crate) struct Provider {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: AgentKind,
    /// Lowercase names this agent uses in `AI_AGENT` or `AGENT`.
    pub slugs: &'static [&'static str],
    /// Returns the matched signal when the provider is detected in `env`.
    pub matches: fn(&Env) -> Option<Hit>,
}

/// The signal that matched, and how strongly it indicates the provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Hit {
    pub signal: &'static str,
    pub confidence: Confidence,
}

impl Hit {
    const fn new(signal: &'static str, confidence: Confidence) -> Self {
        Self { signal, confidence }
    }
}

impl Provider {
    fn detected(&self, hit: Hit, ai_agent: Option<&AiAgent>) -> DetectedAgent {
        DetectedAgent {
            id: self.id,
            name: self.name,
            kind: self.kind,
            confidence: hit.confidence,
            signal: hit.signal,
            version: ai_agent.and_then(|a| a.version.clone()),
            mode: ai_agent.and_then(|a| a.mode.clone()),
            ai_agent: ai_agent.map(|a| a.raw.clone()),
        }
    }

    fn knows_slug(&self, slug: &str) -> bool {
        self.slugs.contains(&slug)
    }
}

/// Returns the first of `keys` that is set to a non-empty value.
fn first_set(env: &Env, keys: &[&'static str], confidence: Confidence) -> Option<Hit> {
    keys.iter()
        .find(|key| env.contains(key))
        .map(|key| Hit::new(key, confidence))
}

/// Returns a hit on `signal` if `condition` holds.
fn hit_if(condition: bool, signal: &'static str, confidence: Confidence) -> Option<Hit> {
    condition.then_some(Hit::new(signal, confidence))
}

use Confidence::{High, Low, Medium};

// ---------------------------------------------------------------------------
// Agent-specific matchers (tier 2)
// ---------------------------------------------------------------------------

fn matches_amp(env: &Env) -> Option<Hit> {
    // Sourcegraph Amp (closed source) sets AMP_CURRENT_THREAD_ID in tool executions.
    // Amp also sets CLAUDECODE, so it must be checked before Claude Code.
    // https://ampcode.com
    first_set(env, &["AMP_CURRENT_THREAD_ID"], Medium)
        .or_else(|| hit_if(env.equals("AGENT", "amp"), "AGENT", Medium))
}

fn matches_cowork(env: &Env) -> Option<Hit> {
    // Claude Cowork runs Claude Code and additionally sets CLAUDE_CODE_IS_COWORK.
    // https://github.com/vercel/vercel/blob/main/packages/detect-agent/src/index.ts
    first_set(env, &["CLAUDE_CODE_IS_COWORK"], Medium)
}

fn matches_claude_code(env: &Env) -> Option<Hit> {
    // Claude Code sets CLAUDE_CODE_CHILD_SESSION=1 only in processes it spawns.
    // CLAUDECODE=1 is also set by IDE extensions in integrated (human) terminals.
    // https://code.claude.com/docs/en/env-vars
    hit_if(
        env.equals("CLAUDE_CODE_CHILD_SESSION", "1"),
        "CLAUDE_CODE_CHILD_SESSION",
        High,
    )
    .or_else(|| first_set(env, &["CLAUDECODE"], Medium))
}

fn matches_kilo(env: &Env) -> Option<Hit> {
    // Kilo Code CLI (an OpenCode fork) sets KILO=1 alongside OPENCODE=1 and AGENT=1.
    // https://github.com/Kilo-Org/kilocode
    hit_if(env.equals("KILO", "1"), "KILO", High)
}

fn matches_opencode(env: &Env) -> Option<Hit> {
    // OpenCode sets OPENCODE=1, AGENT=1 and OPENCODE_SESSION_ID on shell executions.
    // https://github.com/anomalyco/opencode/blob/dev/packages/core/src/tool/plugin/shell.ts
    hit_if(env.equals("OPENCODE", "1"), "OPENCODE", High)
        .or_else(|| first_set(env, &["OPENCODE_SESSION_ID"], High))
}

fn matches_qwen_code(env: &Env) -> Option<Hit> {
    // Qwen Code (Alibaba, Gemini CLI fork) sets QWEN_CODE=1 on every shell exec.
    // https://github.com/QwenLM/qwen-code/blob/main/packages/core/src/services/shellExecutionService.ts
    hit_if(env.equals("QWEN_CODE", "1"), "QWEN_CODE", High)
}

fn matches_gemini_cli(env: &Env) -> Option<Hit> {
    // Gemini CLI (Google) sets GEMINI_CLI=1 on every shell command and MCP server it spawns.
    // https://github.com/google-gemini/gemini-cli/blob/main/packages/core/src/services/shellExecutionService.ts
    hit_if(env.equals("GEMINI_CLI", "1"), "GEMINI_CLI", High)
}

fn matches_codex(env: &Env) -> Option<Hit> {
    // OpenAI Codex injects CODEX_THREAD_ID (and CODEX_SESSION_ID) into every shell command;
    // unified exec adds CODEX_CI=1, sandboxed runs add CODEX_SANDBOX / CODEX_SANDBOX_NETWORK_DISABLED.
    // https://github.com/openai/codex/blob/main/codex-rs/protocol/src/shell_environment.rs
    // https://github.com/openai/codex/blob/main/codex-rs/core/src/spawn.rs
    first_set(
        env,
        &[
            "CODEX_THREAD_ID",
            "CODEX_SESSION_ID",
            "CODEX_CI",
            "CODEX_SANDBOX",
            "CODEX_SANDBOX_NETWORK_DISABLED",
        ],
        High,
    )
}

fn matches_crush(env: &Env) -> Option<Hit> {
    // Crush (Charm) sets CRUSH=1, AGENT=crush and AI_AGENT=crush on every shell exec.
    // https://github.com/charmbracelet/crush/blob/main/internal/shell/shell.go
    hit_if(env.equals("CRUSH", "1"), "CRUSH", High)
        .or_else(|| hit_if(env.equals("AGENT", "crush"), "AGENT", High))
}

fn matches_pi(env: &Env) -> Option<Hit> {
    // Pi coding agent sets PI_CODING_AGENT=true (and AI_AGENT=pi) for the whole process.
    // https://github.com/badlogic/pi-mono/blob/main/packages/coding-agent/src/cli/setup.ts
    hit_if(
        env.equals("PI_CODING_AGENT", "true"),
        "PI_CODING_AGENT",
        High,
    )
}

fn matches_auggie(env: &Env) -> Option<Hit> {
    // Augment Code's Auggie CLI sets AUGMENT_AGENT=1 in the launch-process tool's env.
    // https://docs.augmentcode.com/cli/reference
    hit_if(env.equals("AUGMENT_AGENT", "1"), "AUGMENT_AGENT", High)
}

fn matches_copilot_cli_signal(env: &Env) -> Option<Hit> {
    first_set(
        env,
        &["COPILOT_AGENT_SESSION_ID", "COPILOT_AGENT_JOB_ID"],
        High,
    )
    .or_else(|| hit_if(env.equals("COPILOT_CLI", "1"), "COPILOT_CLI", High))
}

fn matches_copilot_cloud_agent(env: &Env) -> Option<Hit> {
    // GitHub Copilot cloud agent: the Copilot CLI running as a GitHub Actions job.
    matches_copilot_cli_signal(env).filter(|_| env.equals("GITHUB_ACTIONS", "true"))
}

fn matches_copilot_cli(env: &Env) -> Option<Hit> {
    // GitHub Copilot CLI sets COPILOT_CLI=1 (since 0.0.421) and
    // COPILOT_AGENT_SESSION_ID (since 1.0.29) for the commands it runs.
    // https://github.com/github/copilot-cli/blob/main/changelog.md
    matches_copilot_cli_signal(env)
}

fn matches_copilot_vscode(env: &Env) -> Option<Hit> {
    // VS Code agent terminals set COPILOT_AGENT=1, AI_AGENT=github_copilot_vscode_agent,
    // GIT_PAGER=cat and GIT_EDITOR=: (GIT_PAGER alone is the legacy heuristic).
    // https://github.com/microsoft/vscode/blob/main/src/vs/platform/chat/common/aiAgentEnv.ts
    hit_if(env.equals("COPILOT_AGENT", "1"), "COPILOT_AGENT", High).or_else(|| {
        hit_if(
            env.equals("TERM_PROGRAM", "vscode") && env.equals("GIT_PAGER", "cat"),
            "GIT_PAGER",
            Low,
        )
    })
}

fn matches_cursor_agent(env: &Env) -> Option<Hit> {
    // Cursor sets CURSOR_AGENT in agent-run terminal commands (IDE and cursor-agent CLI).
    // https://cursor.com/docs/agent/tools/terminal
    // The CURSOR_TRACE_ID + PAGER override is an older, undocumented heuristic.
    hit_if(env.truthy("CURSOR_AGENT"), "CURSOR_AGENT", High).or_else(|| {
        hit_if(
            env.contains("CURSOR_TRACE_ID") && env.equals("PAGER", "head -n 10000 | cat"),
            "PAGER",
            Low,
        )
    })
}

fn matches_cline(env: &Env) -> Option<Hit> {
    // Cline sets CLINE_ACTIVE=true in the VS Code terminals it drives.
    // https://github.com/cline/cline/blob/main/apps/vscode/src/hosts/vscode/terminal/VscodeTerminalRegistry.ts
    hit_if(env.equals("CLINE_ACTIVE", "true"), "CLINE_ACTIVE", High)
}

fn matches_roo_code(env: &Env) -> Option<Hit> {
    // Roo Code sets ROO_ACTIVE=true in the terminals it drives.
    // https://github.com/RooCodeInc/Roo-Code/blob/main/src/integrations/terminal/Terminal.ts
    hit_if(env.equals("ROO_ACTIVE", "true"), "ROO_ACTIVE", High)
}

fn matches_amazon_q(env: &Env) -> Option<Hit> {
    // Amazon Q Developer CLI appends AmazonQ-For-CLI to AWS_EXECUTION_ENV for executed commands.
    // https://github.com/aws/amazon-q-developer-cli/blob/main/crates/chat-cli/src/cli/chat/consts.rs
    hit_if(
        env.value_contains("AWS_EXECUTION_ENV", "AmazonQ-For-CLI"),
        "AWS_EXECUTION_ENV",
        High,
    )
}

fn matches_openhands(env: &Env) -> Option<Hit> {
    // OpenHands sets AI_AGENT=openhands; its bash session also embeds a JSON
    // marker in the prompt.
    // https://github.com/jdx/mise/blob/main/crates/mise-agent-env/src/lib.rs
    ["PS1", "PROMPT_COMMAND"]
        .into_iter()
        .find(|key| env.value_contains(key, "###PS1JSON###"))
        .map(|key| Hit::new(key, Medium))
}

fn matches_goose(env: &Env) -> Option<Hit> {
    // Goose (Block / AAIF) sets GOOSE_TERMINAL=1 and AGENT=goose for some shell executions.
    // https://github.com/aaif-goose/goose/blob/main/crates/goose/src/agents/retry.rs
    hit_if(env.equals("GOOSE_TERMINAL", "1"), "GOOSE_TERMINAL", High)
        .or_else(|| hit_if(env.equals("AGENT", "goose"), "AGENT", High))
}

fn matches_antigravity(env: &Env) -> Option<Hit> {
    // Antigravity (Google) agent terminals; no vendor documentation, checked by
    // firebase-tools and vercel/detect-agent.
    first_set(env, &["ANTIGRAVITY_AGENT"], Medium)
}

fn matches_junie(env: &Env) -> Option<Hit> {
    // JetBrains Junie; no vendor documentation, checked by std-env and Cypress.
    // https://github.com/unjs/std-env/blob/main/src/agents.ts
    first_set(env, &["JUNIE_DATA", "JUNIE_SHIM_PATH"], Medium)
}

fn matches_kiro_cli(env: &Env) -> Option<Hit> {
    // Kiro CLI (AWS); no vendor documentation, checked by mise and firebase-tools.
    // https://github.com/jdx/mise/blob/main/crates/mise-agent-env/src/lib.rs
    first_set(env, &["KIRO_AGENT_PATH"], Medium)
}

fn matches_trae(env: &Env) -> Option<Hit> {
    // Trae (ByteDance); no vendor documentation, checked by mise.
    // https://github.com/jdx/mise/blob/main/crates/mise-agent-env/src/lib.rs
    first_set(env, &["TRAE_AI_SHELL_ID"], Medium)
}

fn matches_windsurf_cascade(env: &Env) -> Option<Hit> {
    // Windsurf (Codeium) Cascade agent terminals; no vendor documentation,
    // checked by laravel/agent-detector.
    first_set(env, &["WINDSURF_CASCADE_TERMINAL"], Medium)
}

fn matches_jules(env: &Env) -> Option<Hit> {
    // Jules (Google) runs as user "swebot" with HOME=/home/jules; no vendor documentation.
    hit_if(
        env.equals("HOME", "/home/jules") && env.equals("USER", "swebot"),
        "USER",
        Low,
    )
}

fn matches_replit_assistant(env: &Env) -> Option<Hit> {
    // Replit AI Assistant mode: REPL_ID set AND REPLIT_MODE=assistant; no vendor documentation.
    hit_if(
        env.contains("REPL_ID") && env.equals("REPLIT_MODE", "assistant"),
        "REPLIT_MODE",
        Low,
    )
}

fn matches_bolt_agent(env: &Env) -> Option<Hit> {
    // Bolt.new in agent mode: /bin/jsh shell AND npm_config_yes is set; no vendor documentation.
    hit_if(
        env.equals("SHELL", "/bin/jsh") && env.contains("npm_config_yes"),
        "npm_config_yes",
        Low,
    )
}

fn matches_zed_agent(env: &Env) -> Option<Hit> {
    // Zed agent terminals set PAGER to an empty string and GIT_PAGER=cat.
    // https://github.com/zed-industries/zed/blob/main/crates/acp_thread/src/terminal.rs
    hit_if(
        env.equals("TERM_PROGRAM", "zed")
            && env.equals("PAGER", "")
            && env.equals("GIT_PAGER", "cat"),
        "GIT_PAGER",
        High,
    )
}

fn matches_warp_agent(env: &Env) -> Option<Hit> {
    // Warp cloud agent runs; no vendor documentation, checked by mise and am-i-vibing.
    first_set(env, &["OZ_RUN_ID"], Low)
}

fn no_env_signal(_: &Env) -> Option<Hit> {
    // The agent injects no environment variables; it is detected by process
    // ancestry or through AI_AGENT only.
    None
}

fn matches_devin(env: &Env) -> Option<Hit> {
    // Devin's machines point EDITOR at a devin binary; no vendor documentation, checked by std-env.
    hit_if(env.value_contains("EDITOR", "devin"), "EDITOR", Low)
}

// ---------------------------------------------------------------------------
// Ambient matchers (tier 4): IDEs and terminals that also host human sessions
// ---------------------------------------------------------------------------

fn matches_cursor(env: &Env) -> Option<Hit> {
    // Cursor IDE terminal not driven by the agent.
    hit_if(
        env.contains("CURSOR_TRACE_ID") && matches_cursor_agent(env).is_none(),
        "CURSOR_TRACE_ID",
        Low,
    )
}

fn matches_replit(env: &Env) -> Option<Hit> {
    // Replit workspace not in assistant mode.
    hit_if(
        env.contains("REPL_ID") && matches_replit_assistant(env).is_none(),
        "REPL_ID",
        Low,
    )
}

fn matches_bolt(env: &Env) -> Option<Hit> {
    // Bolt.new WebContainer shell not in agent mode.
    hit_if(
        env.equals("SHELL", "/bin/jsh") && matches_bolt_agent(env).is_none(),
        "SHELL",
        Low,
    )
}

fn matches_zed(env: &Env) -> Option<Hit> {
    // Zed terminal not driven by the agent.
    // https://github.com/zed-industries/zed/blob/main/crates/terminal/src/terminal.rs
    hit_if(
        env.equals("TERM_PROGRAM", "zed") && matches_zed_agent(env).is_none(),
        "TERM_PROGRAM",
        Low,
    )
}

fn matches_kiro(env: &Env) -> Option<Hit> {
    // Kiro IDE terminal not driven by the Kiro CLI agent.
    hit_if(
        env.get("TERM_PROGRAM")
            .is_some_and(|v| v.eq_ignore_ascii_case("kiro"))
            && matches_kiro_cli(env).is_none(),
        "TERM_PROGRAM",
        Low,
    )
}

fn matches_windsurf(env: &Env) -> Option<Hit> {
    // Windsurf IDE terminal not driven by Cascade.
    hit_if(
        env.contains("CODEIUM_EDITOR_APP_ROOT") && matches_windsurf_cascade(env).is_none(),
        "CODEIUM_EDITOR_APP_ROOT",
        Low,
    )
}

fn matches_goose_provider(env: &Env) -> Option<Hit> {
    // GOOSE_PROVIDER is Goose configuration that may also live in a user's shell profile.
    first_set(env, &["GOOSE_PROVIDER"], Low)
}

fn matches_warp(env: &Env) -> Option<Hit> {
    // Warp terminal (hybrid: interactive + AI features).
    hit_if(
        env.equals("TERM_PROGRAM", "WarpTerminal"),
        "TERM_PROGRAM",
        Low,
    )
}

// ---------------------------------------------------------------------------
// Provider registry
// ---------------------------------------------------------------------------

const fn agent(
    id: &'static str,
    name: &'static str,
    slugs: &'static [&'static str],
    matches: fn(&Env) -> Option<Hit>,
) -> Provider {
    provider(id, name, AgentKind::Agent, slugs, matches)
}

const fn provider(
    id: &'static str,
    name: &'static str,
    kind: AgentKind,
    slugs: &'static [&'static str],
    matches: fn(&Env) -> Option<Hit>,
) -> Provider {
    Provider {
        id,
        name,
        kind,
        slugs,
        matches,
    }
}

/// Agent-specific providers (tier 2), in detection order: derivatives first.
static AGENTS: &[Provider] = &[
    agent("amp", "Amp", &["amp"], matches_amp),
    agent("cowork", "Claude Cowork", &["cowork"], matches_cowork),
    agent(
        "claude-code",
        "Claude Code",
        &["claude-code", "claude"],
        matches_claude_code,
    ),
    agent(
        "kilo",
        "Kilo Code",
        &["kilo", "kilocode", "kilo-code"],
        matches_kilo,
    ),
    agent("opencode", "OpenCode", &["opencode"], matches_opencode),
    agent(
        "qwen-code",
        "Qwen Code",
        &["qwen-code", "qwen"],
        matches_qwen_code,
    ),
    agent(
        "gemini-cli",
        "Gemini CLI",
        &["gemini-cli", "gemini"],
        matches_gemini_cli,
    ),
    agent(
        "codex",
        "OpenAI Codex",
        &["codex", "codex-cli"],
        matches_codex,
    ),
    agent("crush", "Crush", &["crush"], matches_crush),
    agent("pi", "Pi", &["pi"], matches_pi),
    agent(
        "auggie",
        "Auggie",
        &["auggie", "augment-cli", "augment"],
        matches_auggie,
    ),
    // GitHub Copilot: cloud agent check must come before the local CLI check.
    agent(
        "copilot-cloud-agent",
        "GitHub Copilot Cloud Agent",
        &[],
        matches_copilot_cloud_agent,
    ),
    agent(
        "copilot-cli",
        "GitHub Copilot CLI",
        &[
            "github-copilot-cli",
            "github-copilot",
            "github_copilot",
            "copilot-cli",
            "copilot",
        ],
        matches_copilot_cli,
    ),
    agent(
        "copilot-vscode",
        "GitHub Copilot in VS Code",
        &[
            "github_copilot_vscode_agent",
            "github-copilot-vscode",
            "copilot-vscode",
        ],
        matches_copilot_vscode,
    ),
    agent(
        "cursor-agent",
        "Cursor Agent",
        &["cursor", "cursor-agent", "cursor-cli"],
        matches_cursor_agent,
    ),
    agent("cline", "Cline", &["cline"], matches_cline),
    agent(
        "roo-code",
        "Roo Code",
        &["roo-code", "roo"],
        matches_roo_code,
    ),
    agent(
        "amazon-q",
        "Amazon Q Developer CLI",
        &["amazon-q", "q-cli"],
        matches_amazon_q,
    ),
    agent("openhands", "OpenHands", &["openhands"], matches_openhands),
    agent("goose", "Goose", &["goose"], matches_goose),
    agent(
        "antigravity",
        "Antigravity",
        &["antigravity"],
        matches_antigravity,
    ),
    agent("junie", "Junie", &["junie"], matches_junie),
    agent(
        "kiro-cli",
        "Kiro CLI",
        &["kiro", "kiro-cli"],
        matches_kiro_cli,
    ),
    agent("trae", "Trae", &["trae"], matches_trae),
    agent(
        "windsurf-cascade",
        "Windsurf Cascade",
        &["windsurf", "cascade"],
        matches_windsurf_cascade,
    ),
    agent("jules", "Jules", &["jules"], matches_jules),
    agent(
        "replit-assistant",
        "Replit Assistant",
        &["replit"],
        matches_replit_assistant,
    ),
    agent(
        "bolt-agent",
        "Bolt.new Agent",
        &["bolt"],
        matches_bolt_agent,
    ),
    agent("zed-agent", "Zed Agent", &["zed"], matches_zed_agent),
    agent(
        "warp-agent",
        "Warp Agent",
        &["warp", "oz"],
        matches_warp_agent,
    ),
    agent("devin", "Devin", &["devin"], matches_devin),
    // Aider sets no marker on the commands it runs (aider/run_cmd.py).
    agent("aider", "Aider", &["aider"], no_env_signal),
    agent(
        "factory-droid",
        "Factory Droid",
        &["droid", "factory-droid"],
        no_env_signal,
    ),
    agent("octofriend", "Octofriend", &["octofriend"], no_env_signal),
];

/// Ambient providers (tier 4): IDEs and terminals that also host human sessions.
static AMBIENT: &[Provider] = &[
    provider(
        "cursor",
        "Cursor",
        AgentKind::Interactive,
        &[],
        matches_cursor,
    ),
    provider(
        "replit",
        "Replit",
        AgentKind::Interactive,
        &[],
        matches_replit,
    ),
    provider(
        "bolt",
        "Bolt.new",
        AgentKind::Interactive,
        &[],
        matches_bolt,
    ),
    provider("zed", "Zed", AgentKind::Interactive, &[], matches_zed),
    provider("kiro", "Kiro", AgentKind::Interactive, &[], matches_kiro),
    provider(
        "windsurf",
        "Windsurf",
        AgentKind::Interactive,
        &[],
        matches_windsurf,
    ),
    agent("goose", "Goose", &[], matches_goose_provider),
    provider(
        "warp",
        "Warp Terminal",
        AgentKind::Hybrid,
        &[],
        matches_warp,
    ),
];

/// Returns all known providers, in detection order.
pub(crate) fn all_providers() -> impl Iterator<Item = &'static Provider> {
    AGENTS.iter().chain(AMBIENT)
}

/// Looks up the provider that uses `slug` in `AI_AGENT` / `AGENT`.
fn provider_for_slug(slug: &str) -> Option<&'static Provider> {
    all_providers().find(|p| p.knows_slug(slug))
}

fn unknown(hit: Hit, ai_agent: Option<&AiAgent>) -> DetectedAgent {
    provider(UNKNOWN_ID, "Unknown agent", AgentKind::Agent, &[], |_| None).detected(hit, ai_agent)
}

/// Signals beyond the environment that detection may consult.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Sources<'a> {
    /// Check file-system markers (tier 3).
    pub file_markers: bool,
    /// The ancestry of the inspected process, nearest first (tier 4).
    pub ancestry: &'a [Ancestor],
}

/// Run detection against `env` and `sources`, returning every match in
/// detection order, at most one per agent id.
pub(crate) fn detect_all(env: &Env, sources: Sources<'_>) -> Vec<DetectedAgent> {
    let mut found: Vec<DetectedAgent> = Vec::new();
    let mut push = |agent: DetectedAgent| {
        if !found.iter().any(|f| f.id == agent.id) {
            found.push(agent);
        }
    };

    // Tier 1: AI_AGENT naming an agent.
    let announcement = ai_agent::read(env);
    let named = match &announcement {
        Some(Announcement::Named(named)) => Some(named),
        _ => None,
    };
    let announced = named.map(|named| {
        named
            .candidates
            .iter()
            .find_map(|slug| provider_for_slug(slug))
    });
    let ai_agent_hit = Hit::new(AI_AGENT, High);
    match (named, announced) {
        (Some(named), Some(Some(provider))) => push(provider.detected(ai_agent_hit, Some(named))),
        (Some(named), _) => push(unknown(ai_agent_hit, Some(named))),
        _ => {}
    }
    let announced = announced.flatten();

    // Attaches the AI_AGENT details to the provider it names.
    let detected = |provider: &Provider, hit: Hit| {
        let details = named.filter(|_| announced.is_some_and(|a| a.id == provider.id));
        provider.detected(hit, details)
    };

    // Tier 2: agent-specific signals.
    for provider in AGENTS {
        if let Some(hit) = (provider.matches)(env) {
            push(detected(provider, hit));
        }
    }

    // Tier 3: file markers.
    if sources.file_markers {
        for (provider, hit) in file_marker_hits() {
            push(detected(provider, hit));
        }
    }

    // Tier 4: process ancestry.
    for (provider, hit) in process_hits(sources.ancestry) {
        push(detected(provider, hit));
    }

    // Tier 5: ambient signals.
    for provider in AMBIENT {
        if let Some(hit) = (provider.matches)(env) {
            push(detected(provider, hit));
        }
    }

    // Tier 6: anonymous announcements.
    if let Some(value) = env.get("AGENT").map(str::trim).filter(|v| !v.is_empty()) {
        let slug = value.to_ascii_lowercase();
        match provider_for_slug(&slug) {
            Some(provider) => push(detected(provider, Hit::new("AGENT", Medium))),
            // An unknown AGENT name is too generic to trust; only flag values count.
            None if ai_agent::parse(value) == Some(Announcement::Flag) => {
                push(unknown(Hit::new("AGENT", Low), None))
            }
            None => {}
        }
    }
    if announcement == Some(Announcement::Flag) {
        push(unknown(Hit::new(AI_AGENT, Low), None));
    }

    found
}

/// Checks file-system markers left by agents (cargo feature `fs-markers`).
#[cfg(feature = "fs-markers")]
fn file_marker_hits() -> Vec<(&'static Provider, Hit)> {
    // Devin's machines carry a /opt/.devin marker.
    // https://github.com/vercel/vercel/blob/main/packages/detect-agent/src/index.ts
    const MARKERS: &[(&str, &str)] = &[("devin", "/opt/.devin")];

    MARKERS
        .iter()
        .filter(|(_, path)| std::path::Path::new(path).exists())
        .filter_map(|(id, path)| {
            let provider = all_providers().find(|p| p.id == *id)?;
            Some((provider, Hit::new(path, High)))
        })
        .collect()
}

#[cfg(not(feature = "fs-markers"))]
fn file_marker_hits() -> Vec<(&'static Provider, Hit)> {
    Vec::new()
}

/// A process name that identifies an agent when it appears among the ancestors.
struct ProcessRule {
    /// The executable name, compared exactly against [`Ancestor::candidate_names`].
    process: &'static str,
    /// The provider id.
    id: &'static str,
    /// The reported signal, `process:<name>`.
    signal: &'static str,
}

macro_rules! process_rule {
    ($process:literal => $id:literal) => {
        ProcessRule {
            process: $process,
            id: $id,
            signal: concat!("process:", $process),
        }
    };
}

/// Agent executables recognized in the process ancestry.
///
/// Names are kept to distinctive executables; generic words (e.g. `amp`, `pi`,
/// `goose`) would match unrelated programs. All names are at most 15 bytes,
/// the length Linux truncates process names to.
static PROCESS_RULES: &[ProcessRule] = &[
    // Agents without environment signals (process names as used by am-i-vibing).
    // https://github.com/ascorbic/am-i-vibing/blob/main/packages/am-i-vibing/src/providers.ts
    process_rule!("aider" => "aider"),
    process_rule!("devin" => "devin"),
    process_rule!("droid" => "factory-droid"),
    process_rule!("octofriend" => "octofriend"),
    // Agents that also inject environment variables, for commands run with a
    // scrubbed environment (e.g. `env -i`).
    process_rule!("claude" => "claude-code"),
    process_rule!("codex" => "codex"),
    process_rule!("gemini" => "gemini-cli"),
    process_rule!("qwen" => "qwen-code"),
    process_rule!("crush" => "crush"),
    process_rule!("opencode" => "opencode"),
];

/// Matches `ancestry` against [`PROCESS_RULES`], nearest ancestor first.
fn process_hits(ancestry: &[Ancestor]) -> Vec<(&'static Provider, Hit)> {
    ancestry
        .iter()
        .flat_map(Ancestor::candidate_names)
        .filter_map(|name| PROCESS_RULES.iter().find(|rule| rule.process == name))
        .filter_map(|rule| {
            let provider = all_providers().find(|p| p.id == rule.id)?;
            Some((provider, Hit::new(rule.signal, Medium)))
        })
        .collect()
}

/// Run detection against `env` and `sources`, returning the first match.
pub(crate) fn detect(env: &Env, sources: Sources<'_>) -> Option<DetectedAgent> {
    detect_all(env, sources).into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::EnvBuilder;

    fn env(pairs: &[(&str, &str)]) -> Env {
        pairs
            .iter()
            .fold(EnvBuilder::new(), |b, (k, v)| b.set(k, v))
            .build()
    }

    fn detect_id(pairs: &[(&str, &str)]) -> Option<&'static str> {
        detect(&env(pairs), Sources::default()).map(|a| a.id)
    }

    #[track_caller]
    fn assert_detects(pairs: &[(&str, &str)], id: &str, kind: AgentKind, confidence: Confidence) {
        let agent = detect(&env(pairs), Sources::default())
            .unwrap_or_else(|| panic!("expected {id} for {pairs:?}, got nothing"));
        assert_eq!(agent.id, id, "{pairs:?}");
        assert_eq!(agent.kind, kind, "{pairs:?}");
        assert_eq!(agent.confidence, confidence, "{pairs:?}");
    }

    use AgentKind::{Agent, Hybrid, Interactive};

    #[test]
    fn provider_ids_are_unique_per_tier() {
        for tier in [AGENTS, AMBIENT] {
            let mut ids: Vec<_> = tier.iter().map(|p| p.id).collect();
            ids.sort_unstable();
            let len = ids.len();
            ids.dedup();
            assert_eq!(ids.len(), len);
        }
    }

    #[test]
    fn slugs_are_lowercase_and_unique() {
        let mut slugs: Vec<_> = all_providers().flat_map(|p| p.slugs.iter()).collect();
        assert!(slugs.iter().all(|s| s.to_ascii_lowercase() == **s));
        slugs.sort_unstable();
        let len = slugs.len();
        slugs.dedup();
        assert_eq!(slugs.len(), len);
    }

    // --- Tier 1: AI_AGENT ---

    #[test]
    fn ai_agent_claude_code() {
        let agent = detect(
            &env(&[("AI_AGENT", "claude-code_2-1-312_agent")]),
            Sources::default(),
        )
        .unwrap();
        assert_eq!(agent.id, "claude-code");
        assert_eq!(agent.signal, "AI_AGENT");
        assert_eq!(agent.confidence, High);
        assert_eq!(agent.version.as_deref(), Some("2.1.312"));
        assert_eq!(agent.mode.as_deref(), Some("agent"));
        assert_eq!(agent.ai_agent.as_deref(), Some("claude-code_2-1-312_agent"));
    }

    #[test]
    fn ai_agent_vscode_copilot() {
        assert_eq!(
            detect_id(&[("AI_AGENT", "github_copilot_vscode_agent")]),
            Some("copilot-vscode")
        );
    }

    #[test]
    fn ai_agent_at_version() {
        let agent = detect(&env(&[("AI_AGENT", "devin@1")]), Sources::default()).unwrap();
        assert_eq!(agent.id, "devin");
        assert_eq!(agent.version.as_deref(), Some("1"));
    }

    #[test]
    fn ai_agent_known_simple_names() {
        for (value, id) in [
            ("crush", "crush"),
            ("opencode", "opencode"),
            ("pi", "pi"),
            ("openhands", "openhands"),
            ("github-copilot-cli", "copilot-cli"),
        ] {
            assert_eq!(detect_id(&[("AI_AGENT", value)]), Some(id), "{value}");
        }
    }

    #[test]
    fn ai_agent_unknown_name() {
        let agent = detect(&env(&[("AI_AGENT", "hermes@0.3")]), Sources::default()).unwrap();
        assert!(agent.is_unknown());
        assert_eq!(agent.confidence, High);
        assert_eq!(agent.ai_agent.as_deref(), Some("hermes@0.3"));
        assert_eq!(agent.version.as_deref(), Some("0.3"));
    }

    #[test]
    fn ai_agent_wins_over_vendor_signals() {
        let pairs = [
            ("AI_AGENT", "claude-code_2-1-312_agent"),
            ("CLAUDECODE", "1"),
        ];
        let all = detect_all(&env(&pairs), Sources::default());
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].signal, "AI_AGENT");
    }

    #[test]
    fn ai_agent_details_attach_to_matching_vendor_hit_only() {
        let pairs = [
            ("AI_AGENT", "claude-code_2-1-312_agent"),
            ("CODEX_THREAD_ID", "t"),
        ];
        let all = detect_all(&env(&pairs), Sources::default());
        assert_eq!(all[0].id, "claude-code");
        assert_eq!(all[1].id, "codex");
        assert_eq!(all[1].version, None);
        assert_eq!(all[1].ai_agent, None);
    }

    #[test]
    fn ai_agent_falsy_is_ignored() {
        for value in ["", "0", "false", "off"] {
            assert_eq!(detect_id(&[("AI_AGENT", value)]), None, "{value:?}");
        }
    }

    #[test]
    fn ai_agent_flag_is_anonymous_and_lets_vendor_signals_name_the_agent() {
        assert_detects(&[("AI_AGENT", "1")], UNKNOWN_ID, Agent, Low);
        assert_detects(&[("AI_AGENT", "not valid!")], UNKNOWN_ID, Agent, Low);
        assert_eq!(
            detect_id(&[("AI_AGENT", "1"), ("GEMINI_CLI", "1")]),
            Some("gemini-cli")
        );
    }

    // --- Tier 5: AGENT ---

    #[test]
    fn agent_flag_is_anonymous() {
        assert_detects(&[("AGENT", "1")], UNKNOWN_ID, Agent, Low);
    }

    #[test]
    fn agent_known_name() {
        assert_detects(&[("AGENT", "codex")], "codex", Agent, Medium);
    }

    #[test]
    fn agent_unknown_name_is_ignored() {
        assert_eq!(detect_id(&[("AGENT", "smith")]), None);
    }

    // --- Tier 2: agent-specific signals ---

    #[test]
    fn detects_claude_code() {
        assert_detects(
            &[("CLAUDE_CODE_CHILD_SESSION", "1")],
            "claude-code",
            Agent,
            High,
        );
        assert_detects(&[("CLAUDECODE", "1")], "claude-code", Agent, Medium);
    }

    #[test]
    fn detects_cowork_before_claude_code() {
        assert_eq!(
            detect_id(&[("CLAUDECODE", "1"), ("CLAUDE_CODE_IS_COWORK", "1")]),
            Some("cowork")
        );
    }

    #[test]
    fn detects_amp_before_claude_code() {
        assert_eq!(
            detect_id(&[("AMP_CURRENT_THREAD_ID", "t"), ("CLAUDECODE", "1")]),
            Some("amp")
        );
        assert_eq!(detect_id(&[("AGENT", "amp")]), Some("amp"));
    }

    #[test]
    fn detects_kilo_before_opencode() {
        assert_detects(
            &[("KILO", "1"), ("OPENCODE", "1"), ("AGENT", "1")],
            "kilo",
            Agent,
            High,
        );
    }

    #[test]
    fn detects_opencode() {
        assert_detects(
            &[("OPENCODE", "1"), ("AGENT", "1")],
            "opencode",
            Agent,
            High,
        );
        assert_detects(&[("OPENCODE_SESSION_ID", "s")], "opencode", Agent, High);
    }

    #[test]
    fn does_not_detect_opencode_launcher_vars() {
        assert_eq!(
            detect_id(&[("OPENCODE_BIN_PATH", "/usr/bin/opencode")]),
            None
        );
    }

    #[test]
    fn detects_qwen_before_gemini() {
        assert_eq!(
            detect_id(&[("QWEN_CODE", "1"), ("GEMINI_CLI", "1")]),
            Some("qwen-code")
        );
    }

    #[test]
    fn detects_gemini_cli() {
        assert_detects(&[("GEMINI_CLI", "1")], "gemini-cli", Agent, High);
        assert_eq!(detect_id(&[("GEMINI_CLI", "0")]), None);
    }

    #[test]
    fn detects_codex() {
        for key in [
            "CODEX_THREAD_ID",
            "CODEX_SESSION_ID",
            "CODEX_CI",
            "CODEX_SANDBOX",
            "CODEX_SANDBOX_NETWORK_DISABLED",
        ] {
            assert_detects(&[(key, "1")], "codex", Agent, High);
        }
    }

    #[test]
    fn detects_crush() {
        assert_detects(&[("CRUSH", "1")], "crush", Agent, High);
        assert_detects(&[("AGENT", "crush")], "crush", Agent, High);
    }

    #[test]
    fn detects_pi() {
        assert_detects(&[("PI_CODING_AGENT", "true")], "pi", Agent, High);
    }

    #[test]
    fn detects_auggie() {
        assert_detects(&[("AUGMENT_AGENT", "1")], "auggie", Agent, High);
    }

    #[test]
    fn detects_copilot_cloud_agent() {
        assert_detects(
            &[
                ("COPILOT_AGENT_SESSION_ID", "s"),
                ("GITHUB_ACTIONS", "true"),
            ],
            "copilot-cloud-agent",
            Agent,
            High,
        );
        assert_detects(
            &[("COPILOT_CLI", "1"), ("GITHUB_ACTIONS", "true")],
            "copilot-cloud-agent",
            Agent,
            High,
        );
    }

    #[test]
    fn detects_copilot_cli_outside_actions() {
        assert_detects(&[("COPILOT_CLI", "1")], "copilot-cli", Agent, High);
        assert_detects(
            &[("COPILOT_AGENT_SESSION_ID", "s")],
            "copilot-cli",
            Agent,
            High,
        );
    }

    #[test]
    fn detects_copilot_vscode() {
        assert_detects(&[("COPILOT_AGENT", "1")], "copilot-vscode", Agent, High);
        assert_detects(
            &[("TERM_PROGRAM", "vscode"), ("GIT_PAGER", "cat")],
            "copilot-vscode",
            Agent,
            Low,
        );
    }

    #[test]
    fn detects_cursor_agent() {
        assert_detects(&[("CURSOR_AGENT", "1")], "cursor-agent", Agent, High);
        assert_detects(
            &[("CURSOR_TRACE_ID", "t"), ("PAGER", "head -n 10000 | cat")],
            "cursor-agent",
            Agent,
            Low,
        );
        assert_eq!(detect_id(&[("CURSOR_AGENT", "0")]), None);
    }

    #[test]
    fn detects_cline_and_roo() {
        assert_detects(&[("CLINE_ACTIVE", "true")], "cline", Agent, High);
        assert_detects(&[("ROO_ACTIVE", "true")], "roo-code", Agent, High);
    }

    #[test]
    fn detects_amazon_q() {
        assert_detects(
            &[(
                "AWS_EXECUTION_ENV",
                "CloudShell AmazonQ-For-CLI Version/1.2.3",
            )],
            "amazon-q",
            Agent,
            High,
        );
    }

    #[test]
    fn detects_openhands() {
        assert_detects(
            &[("PS1", "\n###PS1JSON###\n{}\n###PS1END###\n")],
            "openhands",
            Agent,
            Medium,
        );
    }

    #[test]
    fn detects_goose() {
        assert_detects(&[("GOOSE_TERMINAL", "1")], "goose", Agent, High);
        assert_detects(&[("AGENT", "goose")], "goose", Agent, High);
        assert_detects(&[("GOOSE_PROVIDER", "anthropic")], "goose", Agent, Low);
    }

    #[test]
    fn detects_medium_confidence_agents() {
        for (key, id) in [
            ("ANTIGRAVITY_AGENT", "antigravity"),
            ("JUNIE_DATA", "junie"),
            ("JUNIE_SHIM_PATH", "junie"),
            ("KIRO_AGENT_PATH", "kiro-cli"),
            ("TRAE_AI_SHELL_ID", "trae"),
            ("WINDSURF_CASCADE_TERMINAL", "windsurf-cascade"),
        ] {
            assert_detects(&[(key, "x")], id, Agent, Medium);
        }
    }

    #[test]
    fn detects_jules() {
        assert_detects(
            &[("HOME", "/home/jules"), ("USER", "swebot")],
            "jules",
            Agent,
            Low,
        );
        assert_eq!(
            detect_id(&[("HOME", "/home/jules"), ("USER", "notjules")]),
            None
        );
    }

    #[test]
    fn detects_replit() {
        assert_detects(
            &[("REPL_ID", "r"), ("REPLIT_MODE", "assistant")],
            "replit-assistant",
            Agent,
            Low,
        );
        assert_detects(&[("REPL_ID", "r")], "replit", Interactive, Low);
    }

    #[test]
    fn detects_bolt() {
        assert_detects(
            &[("SHELL", "/bin/jsh"), ("npm_config_yes", "true")],
            "bolt-agent",
            Agent,
            Low,
        );
        assert_detects(&[("SHELL", "/bin/jsh")], "bolt", Interactive, Low);
    }

    #[test]
    fn detects_zed() {
        assert_detects(
            &[("TERM_PROGRAM", "zed"), ("PAGER", ""), ("GIT_PAGER", "cat")],
            "zed-agent",
            Agent,
            High,
        );
        // The pre-0.2 heuristic (PAGER=cat) does not match Zed's agent terminal.
        assert_detects(
            &[("TERM_PROGRAM", "zed"), ("PAGER", "cat")],
            "zed",
            Interactive,
            Low,
        );
        assert_detects(&[("TERM_PROGRAM", "zed")], "zed", Interactive, Low);
    }

    #[test]
    fn detects_devin_by_editor() {
        assert_detects(
            &[("EDITOR", "/opt/devin/bin/devin-editor")],
            "devin",
            Agent,
            Low,
        );
    }

    // --- Tier 4: ambient ---

    #[test]
    fn detects_cursor_interactive() {
        assert_detects(&[("CURSOR_TRACE_ID", "t")], "cursor", Interactive, Low);
    }

    #[test]
    fn interactive_variant_is_not_listed_next_to_agent_variant() {
        let all = detect_all(
            &env(&[("CURSOR_TRACE_ID", "t"), ("CURSOR_AGENT", "1")]),
            Sources::default(),
        );
        let ids: Vec<_> = all.iter().map(|a| a.id).collect();
        assert_eq!(ids, ["cursor-agent"]);
    }

    #[test]
    fn detects_kiro_and_windsurf_ides() {
        assert_detects(&[("TERM_PROGRAM", "kiro")], "kiro", Interactive, Low);
        assert_detects(
            &[("CODEIUM_EDITOR_APP_ROOT", "/opt/windsurf")],
            "windsurf",
            Interactive,
            Low,
        );
    }

    #[test]
    fn detects_warp() {
        assert_detects(&[("TERM_PROGRAM", "WarpTerminal")], "warp", Hybrid, Low);
        assert_detects(&[("OZ_RUN_ID", "r")], "warp-agent", Agent, Low);
    }

    #[test]
    fn agent_signals_precede_ambient_ones() {
        assert_eq!(
            detect_id(&[("TERM_PROGRAM", "WarpTerminal"), ("CLAUDECODE", "1")]),
            Some("claude-code")
        );
    }

    // --- Tier 4: process ancestry ---

    fn detect_all_with_ancestry(
        pairs: &[(&str, &str)],
        ancestry: &[Ancestor],
    ) -> Vec<DetectedAgent> {
        detect_all(
            &env(pairs),
            Sources {
                file_markers: false,
                ancestry,
            },
        )
    }

    #[test]
    fn detects_agents_without_env_signals_by_ancestry() {
        for (ancestor, id) in [
            (
                Ancestor::new("aider", ["/usr/bin/python3", "/home/me/.local/bin/aider"]),
                "aider",
            ),
            (
                Ancestor::new("python3", ["python3", "-m", "aider"]),
                "aider",
            ),
            (Ancestor::new("devin", ["devin"]), "devin"),
            (
                Ancestor::new("droid", ["/usr/local/bin/droid"]),
                "factory-droid",
            ),
            (
                Ancestor::new("node", ["node", "/usr/local/bin/octofriend"]),
                "octofriend",
            ),
        ] {
            let all = detect_all_with_ancestry(&[], &[Ancestor::new("bash", ["bash"]), ancestor]);
            assert_eq!(all.len(), 1, "{all:?}");
            assert_eq!(all[0].id, id);
            assert_eq!(all[0].confidence, Medium);
            assert!(all[0].signal.starts_with("process:"), "{}", all[0].signal);
        }
    }

    #[test]
    fn detects_claude_code_by_ancestry_with_scrubbed_env() {
        // Live sample: Claude Code's exe is named after its version, but comm and argv[0] are "claude".
        let ancestry = [
            Ancestor::new("bash", ["/bin/bash", "-c", "..."]),
            Ancestor::new("claude", ["claude"]),
        ];
        let all = detect_all_with_ancestry(&[], &ancestry);
        assert_eq!(all[0].id, "claude-code");
        assert_eq!(all[0].signal, "process:claude");
    }

    #[test]
    fn detects_node_based_agents_by_script_name() {
        let ancestry = [Ancestor::new(
            "node",
            ["node", "--no-warnings", "/usr/local/bin/gemini"],
        )];
        assert_eq!(detect_all_with_ancestry(&[], &ancestry)[0].id, "gemini-cli");
    }

    #[test]
    fn nearest_ancestor_comes_first() {
        let ancestry = [
            Ancestor::new("aider", ["aider"]),
            Ancestor::new("claude", ["claude"]),
        ];
        let ids: Vec<_> = detect_all_with_ancestry(&[], &ancestry)
            .iter()
            .map(|a| a.id)
            .collect();
        assert_eq!(ids, ["aider", "claude-code"]);
    }

    #[test]
    fn env_signals_precede_ancestry_and_ancestry_precedes_ambient() {
        let ancestry = [Ancestor::new("aider", ["aider"])];
        let pairs = [("CODEX_THREAD_ID", "t"), ("TERM_PROGRAM", "WarpTerminal")];
        let ids: Vec<_> = detect_all_with_ancestry(&pairs, &ancestry)
            .iter()
            .map(|a| a.id)
            .collect();
        assert_eq!(ids, ["codex", "aider", "warp"]);
    }

    #[test]
    fn same_agent_from_env_and_ancestry_is_listed_once() {
        let ancestry = [Ancestor::new("claude", ["claude"])];
        let all = detect_all_with_ancestry(&[("CLAUDE_CODE_CHILD_SESSION", "1")], &ancestry);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].signal, "CLAUDE_CODE_CHILD_SESSION");
    }

    #[test]
    fn unrelated_and_partial_process_names_do_not_match() {
        let ancestry = [
            Ancestor::new("bash", ["/bin/bash"]),
            Ancestor::new("vim", ["vim", "/home/me/aider-notes.md"]),
            Ancestor::new("codex-helper", ["/opt/codex-helper"]),
            Ancestor::new("systemd", ["/usr/lib/systemd/systemd", "--user"]),
        ];
        assert!(detect_all_with_ancestry(&[], &ancestry).is_empty());
    }

    #[test]
    fn process_rules_reference_known_providers_and_fit_comm() {
        for rule in PROCESS_RULES {
            assert!(all_providers().any(|p| p.id == rule.id), "{}", rule.id);
            assert!(rule.process.len() <= 15, "{}", rule.process);
        }
    }

    #[test]
    fn agents_without_env_signal_are_detected_via_ai_agent() {
        assert_eq!(detect_id(&[("AI_AGENT", "aider")]), Some("aider"));
        assert_eq!(detect_id(&[("AI_AGENT", "droid")]), Some("factory-droid"));
    }

    // --- Removed and unrelated signals ---

    #[test]
    fn removed_signals_no_longer_match() {
        for (key, value) in [
            ("AIDER_API_KEY", "sk-abc"),
            ("ANTIGRAVITY_PROJECT_ID", "p"),
            ("OPENCODE_SERVER", "http://localhost"),
        ] {
            assert_eq!(detect_id(&[(key, value)]), None, "{key}");
        }
    }

    #[test]
    fn no_detection_in_empty_or_unrelated_env() {
        assert_eq!(detect_id(&[]), None);
        assert_eq!(detect_id(&[("PATH", "/usr/bin:/bin")]), None);
    }

    #[test]
    fn detect_all_lists_nested_agents_once() {
        let pairs = [
            ("CLAUDECODE", "1"),
            ("CODEX_THREAD_ID", "t"),
            ("TERM_PROGRAM", "WarpTerminal"),
        ];
        let ids: Vec<_> = detect_all(&env(&pairs), Sources::default())
            .iter()
            .map(|a| a.id)
            .collect();
        assert_eq!(ids, ["claude-code", "codex", "warp"]);
    }
}
