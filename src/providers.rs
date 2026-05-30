use crate::env::Env;
use crate::types::{AgentKind, DetectedAgent};

/// A provider definition: a static description of an AI coding agent and
/// the function used to detect its presence in the environment.
pub(crate) struct Provider {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: AgentKind,
    /// Returns `true` when the provider is detected in `env`.
    pub matches: fn(&Env) -> bool,
}

impl Provider {
    fn detect(&self, env: &Env) -> Option<DetectedAgent> {
        if (self.matches)(env) {
            Some(DetectedAgent {
                id: self.id,
                name: self.name,
                kind: self.kind.clone(),
            })
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Individual matcher functions
// ---------------------------------------------------------------------------

fn matches_opencode(env: &Env) -> bool {
    // OpenCode sets any of these variables on shell executions.
    // https://opencode.ai
    env.contains("OPENCODE")
        || env.contains("OPENCODE_BIN_PATH")
        || env.contains("OPENCODE_SERVER")
        || env.contains("OPENCODE_APP_INFO")
        || env.contains("OPENCODE_MODES")
}

fn matches_jules(env: &Env) -> bool {
    // Jules (Google) runs as user "swebot" with HOME=/home/jules.
    env.equals("HOME", "/home/jules") && env.equals("USER", "swebot")
}

fn matches_claude_code(env: &Env) -> bool {
    // Claude Code (Anthropic) sets CLAUDECODE in spawned shells.
    // https://docs.anthropic.com/en/docs/claude-code
    env.contains("CLAUDECODE")
}

fn matches_cursor_agent(env: &Env) -> bool {
    // Cursor in agent mode sets CURSOR_TRACE_ID AND overrides PAGER.
    // The PAGER override is the distinguishing signal for the automated agent
    // versus an interactive Cursor terminal session.
    env.contains("CURSOR_TRACE_ID") && env.equals("PAGER", "head -n 10000 | cat")
}

fn matches_cursor(env: &Env) -> bool {
    // Cursor IDE interactive terminal: CURSOR_TRACE_ID without the PAGER override.
    env.contains("CURSOR_TRACE_ID") && !env.equals("PAGER", "head -n 10000 | cat")
}

fn matches_antigravity(env: &Env) -> bool {
    // Antigravity coding agent.
    env.contains("ANTIGRAVITY_AGENT") || env.contains("ANTIGRAVITY_PROJECT_ID")
}

fn matches_gemini_cli(env: &Env) -> bool {
    // Gemini CLI (Google) sets GEMINI_CLI=1 on every shell command and MCP server it spawns.
    // https://github.com/google-gemini/gemini-cli
    env.equals("GEMINI_CLI", "1")
}

fn matches_codex(env: &Env) -> bool {
    // OpenAI Codex injects CODEX_THREAD_ID (the session conversation id) into every shell command.
    // https://github.com/openai/codex/blob/main/codex-rs/protocol/src/shell_environment.rs
    env.contains("CODEX_THREAD_ID")
}

fn matches_replit_assistant(env: &Env) -> bool {
    // Replit AI Assistant mode: REPL_ID set AND REPLIT_MODE=assistant.
    env.contains("REPL_ID") && env.equals("REPLIT_MODE", "assistant")
}

fn matches_replit(env: &Env) -> bool {
    // Replit interactive environment: REPL_ID without the assistant override.
    env.contains("REPL_ID") && !env.equals("REPLIT_MODE", "assistant")
}

fn matches_aider(env: &Env) -> bool {
    // Aider sets AIDER_API_KEY in its shell environment.
    // https://aider.chat
    env.contains("AIDER_API_KEY")
}

fn matches_bolt_agent(env: &Env) -> bool {
    // Bolt.new in agent mode: /bin/jsh shell AND npm_config_yes is set.
    env.equals("SHELL", "/bin/jsh") && env.contains("npm_config_yes")
}

fn matches_bolt(env: &Env) -> bool {
    // Bolt.new interactive environment: /bin/jsh shell WITHOUT npm_config_yes.
    env.equals("SHELL", "/bin/jsh") && !env.contains("npm_config_yes")
}

fn matches_zed_agent(env: &Env) -> bool {
    // Zed editor in agent mode: TERM_PROGRAM=zed AND PAGER=cat (disables pagination).
    env.equals("TERM_PROGRAM", "zed") && env.equals("PAGER", "cat")
}

fn matches_zed(env: &Env) -> bool {
    // Zed editor interactive: TERM_PROGRAM=zed WITHOUT the PAGER override.
    env.equals("TERM_PROGRAM", "zed") && !env.equals("PAGER", "cat")
}

fn matches_windsurf(env: &Env) -> bool {
    // Windsurf (Codeium) sets CODEIUM_EDITOR_APP_ROOT.
    // https://windsurf.com
    env.contains("CODEIUM_EDITOR_APP_ROOT")
}

fn matches_crush(env: &Env) -> bool {
    // Crush (Charm) sets CRUSH=1 (and AGENT=crush, AI_AGENT=crush) on every shell exec.
    // https://github.com/charmbracelet/crush
    env.equals("CRUSH", "1") || env.equals("AGENT", "crush") || env.equals("AI_AGENT", "crush")
}

fn matches_amp(env: &Env) -> bool {
    // Sourcegraph Amp injects AGENT=amp or AMP_CURRENT_THREAD_ID into shell tool executions.
    // https://ampcode.com
    env.contains("AMP_CURRENT_THREAD_ID") || env.equals("AGENT", "amp")
}

fn matches_auggie(env: &Env) -> bool {
    // Augment Code's Auggie CLI sets AUGMENT_AGENT=1 in the shell tool's env.
    // https://augmentcode.com
    env.equals("AUGMENT_AGENT", "1")
}

fn matches_qwen_code(env: &Env) -> bool {
    // Qwen Code (Alibaba, Gemini CLI fork) sets QWEN_CODE=1 on every shell exec.
    env.equals("QWEN_CODE", "1")
}

fn matches_copilot_cloud_agent(env: &Env) -> bool {
    // GitHub Copilot Cloud Agent (Copilot coding agent running as a GitHub Actions job).
    // It sets COPILOT_AGENT_SESSION_ID and/or COPILOT_CLI=1 together with GITHUB_ACTIONS.
    (env.contains("COPILOT_AGENT_SESSION_ID")
        || env.contains("COPILOT_AGENT_JOB_ID")
        || env.equals("COPILOT_CLI", "1"))
        && env.equals("GITHUB_ACTIONS", "true")
}

fn matches_copilot_vscode(env: &Env) -> bool {
    // GitHub Copilot agent mode inside VS Code: TERM_PROGRAM=vscode AND GIT_PAGER=cat.
    // The GIT_PAGER override is the distinguishing signal from an ordinary VS Code terminal.
    env.equals("TERM_PROGRAM", "vscode") && env.equals("GIT_PAGER", "cat")
}

fn matches_warp(env: &Env) -> bool {
    // Warp terminal (hybrid: interactive + AI features).
    env.equals("TERM_PROGRAM", "WarpTerminal")
}

// ---------------------------------------------------------------------------
// Provider registry
// ---------------------------------------------------------------------------

/// Returns the ordered list of known AI coding agent providers.
///
/// Detection is attempted in this order; the **first match wins**.  More
/// specific checks (e.g. "agent with extra env flag") are placed before their
/// less specific counterparts (e.g. "same tool, interactive mode").
pub(crate) fn all_providers() -> &'static [Provider] {
    static PROVIDERS: std::sync::OnceLock<Vec<Provider>> = std::sync::OnceLock::new();
    PROVIDERS.get_or_init(|| {
        vec![
            // --- Agents that have unique, unambiguous environment signals ---
            Provider {
                id: "opencode",
                name: "OpenCode",
                kind: AgentKind::Agent,
                matches: matches_opencode,
            },
            Provider {
                id: "jules",
                name: "Jules",
                kind: AgentKind::Agent,
                matches: matches_jules,
            },
            Provider {
                id: "claude-code",
                name: "Claude Code",
                kind: AgentKind::Agent,
                matches: matches_claude_code,
            },
            Provider {
                id: "gemini-cli",
                name: "Gemini CLI",
                kind: AgentKind::Agent,
                matches: matches_gemini_cli,
            },
            Provider {
                id: "codex",
                name: "OpenAI Codex",
                kind: AgentKind::Agent,
                matches: matches_codex,
            },
            Provider {
                id: "aider",
                name: "Aider",
                kind: AgentKind::Agent,
                matches: matches_aider,
            },
            Provider {
                id: "windsurf",
                name: "Windsurf",
                kind: AgentKind::Agent,
                matches: matches_windsurf,
            },
            Provider {
                id: "antigravity",
                name: "Antigravity",
                kind: AgentKind::Agent,
                matches: matches_antigravity,
            },
            Provider {
                id: "crush",
                name: "Crush",
                kind: AgentKind::Agent,
                matches: matches_crush,
            },
            Provider {
                id: "amp",
                name: "Amp",
                kind: AgentKind::Agent,
                matches: matches_amp,
            },
            Provider {
                id: "auggie",
                name: "Auggie",
                kind: AgentKind::Agent,
                matches: matches_auggie,
            },
            Provider {
                id: "qwen-code",
                name: "Qwen Code",
                kind: AgentKind::Agent,
                matches: matches_qwen_code,
            },
            // GitHub Copilot: cloud agent check must come before VS Code check
            Provider {
                id: "copilot-cloud-agent",
                name: "GitHub Copilot Cloud Agent",
                kind: AgentKind::Agent,
                matches: matches_copilot_cloud_agent,
            },
            // --- Agents that share an env var with an interactive mode ---
            // More specific (agent) checks first, less specific (interactive) after.
            Provider {
                id: "cursor-agent",
                name: "Cursor Agent",
                kind: AgentKind::Agent,
                matches: matches_cursor_agent,
            },
            Provider {
                id: "cursor",
                name: "Cursor",
                kind: AgentKind::Interactive,
                matches: matches_cursor,
            },
            Provider {
                id: "replit-assistant",
                name: "Replit Assistant",
                kind: AgentKind::Agent,
                matches: matches_replit_assistant,
            },
            Provider {
                id: "replit",
                name: "Replit",
                kind: AgentKind::Interactive,
                matches: matches_replit,
            },
            Provider {
                id: "bolt-agent",
                name: "Bolt.new Agent",
                kind: AgentKind::Agent,
                matches: matches_bolt_agent,
            },
            Provider {
                id: "bolt",
                name: "Bolt.new",
                kind: AgentKind::Interactive,
                matches: matches_bolt,
            },
            Provider {
                id: "zed-agent",
                name: "Zed Agent",
                kind: AgentKind::Agent,
                matches: matches_zed_agent,
            },
            Provider {
                id: "zed",
                name: "Zed",
                kind: AgentKind::Interactive,
                matches: matches_zed,
            },
            Provider {
                id: "copilot-vscode",
                name: "GitHub Copilot in VS Code",
                kind: AgentKind::Agent,
                matches: matches_copilot_vscode,
            },
            // --- Hybrid environments ---
            Provider {
                id: "warp",
                name: "Warp Terminal",
                kind: AgentKind::Hybrid,
                matches: matches_warp,
            },
        ]
    })
}

/// Run detection against the provided `env`, returning the first matching provider.
pub(crate) fn detect(env: &Env) -> Option<DetectedAgent> {
    all_providers()
        .iter()
        .find_map(|provider| provider.detect(env))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::EnvBuilder;

    fn env_with(key: &str, value: &str) -> Env {
        EnvBuilder::new().set(key, value).build()
    }

    fn env_with2(k1: &str, v1: &str, k2: &str, v2: &str) -> Env {
        EnvBuilder::new().set(k1, v1).set(k2, v2).build()
    }

    #[test]
    fn detects_opencode() {
        let env = env_with("OPENCODE", "1");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "opencode");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_opencode_by_bin_path() {
        let env = env_with("OPENCODE_BIN_PATH", "/usr/local/bin/opencode");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "opencode");
    }

    #[test]
    fn detects_jules() {
        let env = env_with2("HOME", "/home/jules", "USER", "swebot");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "jules");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn does_not_detect_jules_wrong_user() {
        let env = env_with2("HOME", "/home/jules", "USER", "notjules");
        assert!(detect(&env).is_none());
    }

    #[test]
    fn detects_claude_code() {
        let env = env_with("CLAUDECODE", "1");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "claude-code");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_gemini_cli() {
        let env = env_with("GEMINI_CLI", "1");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "gemini-cli");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn does_not_detect_gemini_cli_wrong_value() {
        let env = env_with("GEMINI_CLI", "0");
        assert!(detect(&env).is_none());
    }

    #[test]
    fn detects_codex() {
        let env = env_with("CODEX_THREAD_ID", "thread-abc-123");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "codex");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_aider() {
        let env = env_with("AIDER_API_KEY", "sk-abc123");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "aider");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_windsurf() {
        let env = env_with("CODEIUM_EDITOR_APP_ROOT", "/opt/windsurf");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "windsurf");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_antigravity_by_agent_var() {
        let env = env_with("ANTIGRAVITY_AGENT", "true");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "antigravity");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_antigravity_by_project_id() {
        let env = env_with("ANTIGRAVITY_PROJECT_ID", "proj-abc");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "antigravity");
    }

    #[test]
    fn detects_crush_by_crush_var() {
        let env = env_with("CRUSH", "1");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "crush");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_crush_by_agent_var() {
        let env = env_with("AGENT", "crush");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "crush");
    }

    #[test]
    fn detects_amp_by_thread_id() {
        let env = env_with("AMP_CURRENT_THREAD_ID", "thread-xyz");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "amp");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_amp_by_agent_var() {
        let env = env_with("AGENT", "amp");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "amp");
    }

    #[test]
    fn detects_auggie() {
        let env = env_with("AUGMENT_AGENT", "1");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "auggie");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_qwen_code() {
        let env = env_with("QWEN_CODE", "1");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "qwen-code");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_copilot_cloud_agent() {
        let env = env_with2(
            "COPILOT_AGENT_SESSION_ID",
            "sess-abc",
            "GITHUB_ACTIONS",
            "true",
        );
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "copilot-cloud-agent");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_copilot_cloud_agent_by_cli_flag() {
        let env = env_with2("COPILOT_CLI", "1", "GITHUB_ACTIONS", "true");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "copilot-cloud-agent");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn does_not_detect_copilot_cloud_without_github_actions() {
        let env = env_with("COPILOT_AGENT_SESSION_ID", "sess-abc");
        assert!(detect(&env).is_none());
    }

    #[test]
    fn detects_cursor_agent() {
        let env = env_with2(
            "CURSOR_TRACE_ID",
            "trace-abc",
            "PAGER",
            "head -n 10000 | cat",
        );
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "cursor-agent");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_cursor_interactive() {
        let env = env_with("CURSOR_TRACE_ID", "trace-abc");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "cursor");
        assert_eq!(result.kind, AgentKind::Interactive);
    }

    #[test]
    fn detects_replit_assistant() {
        let env = env_with2("REPL_ID", "repl-123", "REPLIT_MODE", "assistant");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "replit-assistant");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_replit_interactive() {
        let env = env_with("REPL_ID", "repl-123");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "replit");
        assert_eq!(result.kind, AgentKind::Interactive);
    }

    #[test]
    fn detects_bolt_agent() {
        let env = env_with2("SHELL", "/bin/jsh", "npm_config_yes", "true");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "bolt-agent");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_bolt_interactive() {
        let env = env_with("SHELL", "/bin/jsh");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "bolt");
        assert_eq!(result.kind, AgentKind::Interactive);
    }

    #[test]
    fn detects_zed_agent() {
        let env = env_with2("TERM_PROGRAM", "zed", "PAGER", "cat");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "zed-agent");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_zed_interactive() {
        let env = env_with("TERM_PROGRAM", "zed");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "zed");
        assert_eq!(result.kind, AgentKind::Interactive);
    }

    #[test]
    fn detects_copilot_vscode_agent() {
        let env = env_with2("TERM_PROGRAM", "vscode", "GIT_PAGER", "cat");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "copilot-vscode");
        assert_eq!(result.kind, AgentKind::Agent);
    }

    #[test]
    fn detects_warp() {
        let env = env_with("TERM_PROGRAM", "WarpTerminal");
        let result = detect(&env).unwrap();
        assert_eq!(result.id, "warp");
        assert_eq!(result.kind, AgentKind::Hybrid);
    }

    #[test]
    fn no_detection_in_empty_env() {
        let env = Env::from_map(std::collections::HashMap::new());
        assert!(detect(&env).is_none());
    }

    #[test]
    fn no_detection_for_unrelated_env() {
        let env = env_with("PATH", "/usr/bin:/bin");
        assert!(detect(&env).is_none());
    }
}
