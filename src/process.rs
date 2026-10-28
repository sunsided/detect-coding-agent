//! Process-ancestry signals: agents that are visible only as a parent process.
//!
//! Some agents inject no environment variables into the commands they run
//! (e.g. Aider, Devin, Factory Droid, Octofriend). The only way to detect them
//! is to walk up the process tree and look for the agent's executable.
//!
//! Reading the ancestry of the current process requires the `process-detection`
//! feature and is currently implemented for Linux only (via `/proc`, without
//! spawning subprocesses). On other platforms the ancestry is empty.
//! [`crate::detect_with_env_and_ancestry`] accepts a caller-supplied ancestry
//! on every platform.

/// A process in the ancestry of the current process.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Ancestor {
    /// The process id, if known.
    pub pid: Option<u32>,

    /// The kernel's name for the process (on Linux, `/proc/<pid>/comm`,
    /// truncated to 15 bytes).
    pub name: String,

    /// The command line, starting with `argv[0]`.
    pub args: Vec<String>,
}

impl Ancestor {
    /// Creates an ancestor from its process name and command line.
    ///
    /// ```rust
    /// use detect_coding_agent::Ancestor;
    ///
    /// let aider = Ancestor::new("aider", ["/usr/bin/python3", "/home/me/.local/bin/aider"]);
    /// assert_eq!(aider.name, "aider");
    /// ```
    pub fn new<I, S>(name: impl Into<String>, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            pid: None,
            name: name.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    /// Returns the executable names this process could be known by, in order:
    /// the process name, the basename of `argv[0]`, and, for interpreters such
    /// as `node` or `python3`, the script or module being run.
    pub(crate) fn candidate_names(&self) -> Vec<&str> {
        let mut names = vec![self.name.as_str()];
        names.extend(self.args.first().map(|arg| basename(arg)));

        if names.iter().any(|name| is_interpreter(name)) {
            names.extend(script_name(self.args.get(1..).unwrap_or_default()));
        }

        let mut unique = Vec::with_capacity(names.len());
        for name in names {
            if !name.is_empty() && !unique.contains(&name) {
                unique.push(name);
            }
        }
        unique
    }
}

/// Interpreters whose script argument names the actual program.
fn is_interpreter(name: &str) -> bool {
    matches!(name, "node" | "nodejs" | "bun" | "deno")
        || name
            .strip_prefix("python")
            .is_some_and(|version| version.chars().all(|c| c.is_ascii_digit() || c == '.'))
}

/// Finds the script (or `-m` module) in an interpreter's arguments.
fn script_name(args: &[String]) -> Option<&str> {
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-m" => return args.next().map(|module| module.as_str()),
            // `deno run [flags] script` / `bun run script`
            "run" => continue,
            flag if flag.starts_with('-') => continue,
            script => return Some(strip_script_extension(basename(script))),
        }
    }
    None
}

fn basename(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn strip_script_extension(name: &str) -> &str {
    [".js", ".mjs", ".cjs", ".ts", ".py"]
        .iter()
        .find_map(|ext| name.strip_suffix(ext))
        .unwrap_or(name)
}

/// Returns the ancestry of the current process, nearest ancestor first.
///
/// The result is computed once and cached: a process's ancestry does not
/// change, except when it is reparented after its parent exits.
#[cfg(feature = "process-detection")]
pub(crate) fn current_ancestry() -> &'static [Ancestor] {
    static ANCESTRY: std::sync::OnceLock<Vec<Ancestor>> = std::sync::OnceLock::new();
    ANCESTRY.get_or_init(read_ancestry)
}

/// Maximum number of ancestors to visit.
#[cfg(all(feature = "process-detection", target_os = "linux"))]
const MAX_DEPTH: usize = 32;

#[cfg(all(feature = "process-detection", target_os = "linux"))]
fn read_ancestry() -> Vec<Ancestor> {
    linux::read_ancestry(std::os::unix::process::parent_id(), MAX_DEPTH)
}

#[cfg(all(feature = "process-detection", not(target_os = "linux")))]
fn read_ancestry() -> Vec<Ancestor> {
    Vec::new()
}

/// Reads process ancestry from `/proc`.
#[cfg(any(test, all(feature = "process-detection", target_os = "linux")))]
#[cfg_attr(
    not(all(feature = "process-detection", target_os = "linux")),
    allow(dead_code)
)]
mod linux {
    use super::Ancestor;

    /// Walks up from `pid`, stopping at init, an unreadable process
    /// (e.g. `hidepid`, or the edge of a PID namespace) or after `max_depth` steps.
    pub fn read_ancestry(mut pid: u32, max_depth: usize) -> Vec<Ancestor> {
        let mut ancestry = Vec::new();
        while pid > 1 && ancestry.len() < max_depth {
            let Some(ppid) = read_parent(pid) else {
                break;
            };
            ancestry.push(read_process(pid));
            pid = ppid;
        }
        ancestry
    }

    fn read_parent(pid: u32) -> Option<u32> {
        parse_stat_ppid(&std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?)
    }

    fn read_process(pid: u32) -> Ancestor {
        let name = std::fs::read_to_string(format!("/proc/{pid}/comm")).unwrap_or_default();
        let args = std::fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default();
        Ancestor {
            pid: Some(pid),
            name: name.trim_end_matches('\n').to_owned(),
            args: parse_cmdline(&args),
        }
    }

    /// Extracts the parent pid from `/proc/<pid>/stat`.
    ///
    /// The second field is the process name in parentheses and may itself
    /// contain spaces and parentheses, so parsing starts after the last `)`.
    pub fn parse_stat_ppid(stat: &str) -> Option<u32> {
        let (_, rest) = stat.rsplit_once(')')?;
        // Fields after the name: state, ppid, ...
        rest.split_whitespace().nth(1)?.parse().ok()
    }

    /// Splits the NUL-separated `/proc/<pid>/cmdline`.
    pub fn parse_cmdline(cmdline: &[u8]) -> Vec<String> {
        cmdline
            .split(|&b| b == 0)
            .filter(|arg| !arg.is_empty())
            .map(|arg| String::from_utf8_lossy(arg).into_owned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::linux::{parse_cmdline, parse_stat_ppid};
    use super::*;

    #[test]
    fn parses_ppid_from_stat() {
        assert_eq!(
            parse_stat_ppid("4146610 (claude) S 4145314 4146610 ..."),
            Some(4145314)
        );
        // Process names may contain spaces and parentheses.
        assert_eq!(parse_stat_ppid("42 (a (b) c) R 7 42 42 0"), Some(7));
        assert_eq!(parse_stat_ppid("garbage"), None);
    }

    #[test]
    fn parses_cmdline() {
        assert_eq!(
            parse_cmdline(b"node\0/usr/bin/gemini\0-y\0"),
            ["node", "/usr/bin/gemini", "-y"]
        );
        assert!(parse_cmdline(b"").is_empty());
    }

    #[test]
    fn candidate_names_of_native_binary() {
        // Claude Code's executable is named after its version; comm and argv[0] are not.
        let claude = Ancestor::new("claude", ["claude"]);
        assert_eq!(claude.candidate_names(), ["claude"]);
    }

    #[test]
    fn candidate_names_of_node_script() {
        let gemini = Ancestor::new("node", ["node", "--no-warnings", "/usr/local/bin/gemini"]);
        assert_eq!(gemini.candidate_names(), ["node", "gemini"]);

        let codex = Ancestor::new("node", ["/usr/bin/node", "/opt/codex/bin/codex.js"]);
        assert_eq!(codex.candidate_names(), ["node", "codex"]);
    }

    #[test]
    fn candidate_names_of_python_module() {
        let aider = Ancestor::new("python3", ["python3.12", "-m", "aider"]);
        assert_eq!(aider.candidate_names(), ["python3", "python3.12", "aider"]);
    }

    #[test]
    fn candidate_names_of_shebang_script() {
        // A shebang script keeps its own name as comm, while argv[0] is the interpreter.
        let aider = Ancestor::new("aider", ["/usr/bin/python3", "/home/me/.local/bin/aider"]);
        assert_eq!(aider.candidate_names(), ["aider", "python3"]);
    }

    #[test]
    fn interpreter_without_script() {
        let repl = Ancestor::new("node", ["node"]);
        assert_eq!(repl.candidate_names(), ["node"]);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn reads_own_ancestry_on_linux() {
        let ancestry = super::linux::read_ancestry(std::os::unix::process::parent_id(), 32);
        assert!(!ancestry.is_empty(), "the test runner should have a parent");
        assert_eq!(ancestry[0].pid, Some(std::os::unix::process::parent_id()));
    }
}
