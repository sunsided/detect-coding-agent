use std::collections::HashMap;

/// A snapshot of the process environment used during detection.
///
/// Using this abstraction allows detection logic to be tested
/// without manipulating real environment variables.
pub struct Env(HashMap<String, String>);

impl Env {
    /// Captures the current process environment.
    ///
    /// Variables whose name or value is not valid UTF-8 are skipped
    /// (no known agent signal relies on them).
    pub fn current() -> Self {
        Self(
            std::env::vars_os()
                .filter_map(|(k, v)| Some((k.into_string().ok()?, v.into_string().ok()?)))
                .collect(),
        )
    }

    /// Creates an `Env` from an existing [`HashMap`].
    pub fn from_map(map: HashMap<String, String>) -> Self {
        Self(map)
    }

    /// Returns `true` if the environment contains `key` with any (non-empty) value.
    pub fn contains(&self, key: &str) -> bool {
        self.get(key).is_some_and(|v| !v.is_empty())
    }

    /// Returns `true` if `key` is set to exactly `expected` (which may be empty).
    pub fn equals(&self, key: &str, expected: &str) -> bool {
        self.get(key).is_some_and(|v| v == expected)
    }

    /// Returns `true` if `key` is set to a value that is not empty and not one of
    /// `0`, `false`, `no` or `off` (case-insensitive, surrounding whitespace ignored).
    pub fn truthy(&self, key: &str) -> bool {
        self.get(key).is_some_and(is_truthy)
    }

    /// Returns `true` if the value of `key` contains `needle`.
    pub fn value_contains(&self, key: &str, needle: &str) -> bool {
        self.get(key).is_some_and(|v| v.contains(needle))
    }

    /// Returns the value associated with `key`, if present.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|s| s.as_str())
    }
}

/// See [`Env::truthy`].
pub fn is_truthy(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && !["0", "false", "no", "off"]
            .iter()
            .any(|falsy| value.eq_ignore_ascii_case(falsy))
}

/// Convenience builder for constructing test environments.
#[cfg(test)]
pub struct EnvBuilder(HashMap<String, String>);

#[cfg(test)]
impl EnvBuilder {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn set(mut self, key: &str, value: &str) -> Self {
        self.0.insert(key.to_owned(), value.to_owned());
        self
    }

    pub fn build(self) -> Env {
        Env::from_map(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truthy_rejects_falsy_values() {
        for value in ["", " ", "0", "false", "FALSE", "no", "Off", " off "] {
            assert!(!is_truthy(value), "{value:?} should not be truthy");
        }
        for value in ["1", "true", "yes", "claude-code", "devin@1"] {
            assert!(is_truthy(value), "{value:?} should be truthy");
        }
    }

    #[test]
    fn equals_distinguishes_empty_from_unset() {
        let env = EnvBuilder::new().set("PAGER", "").build();
        assert!(env.equals("PAGER", ""));
        assert!(!env.contains("PAGER"));
        assert!(!env.equals("GIT_PAGER", ""));
    }

    #[test]
    fn current_does_not_panic() {
        let _ = Env::current();
    }
}
