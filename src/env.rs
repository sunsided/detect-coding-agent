use std::collections::HashMap;

/// A snapshot of the process environment used during detection.
///
/// Using this abstraction allows detection logic to be tested
/// without manipulating real environment variables.
pub struct Env(HashMap<String, String>);

impl Env {
    /// Captures the current process environment.
    pub fn current() -> Self {
        Self(std::env::vars().collect())
    }

    /// Creates an `Env` from an existing [`HashMap`].
    pub fn from_map(map: HashMap<String, String>) -> Self {
        Self(map)
    }

    /// Returns `true` if the environment contains `key` with any (non-empty) value.
    pub fn contains(&self, key: &str) -> bool {
        self.0.get(key).map(|v| !v.is_empty()).unwrap_or(false)
    }

    /// Returns `true` if `key` is set to exactly `expected`.
    pub fn equals(&self, key: &str, expected: &str) -> bool {
        self.0.get(key).map(|v| v == expected).unwrap_or(false)
    }

    /// Returns the value associated with `key`, if present.
    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|s| s.as_str())
    }
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
