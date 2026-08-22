//! Check failure model.
//!
//! Per SPINE §4, failure output is one violation per line,
//! `RULE path:line message`, exit 1. `Violation` is that line.

/// A single check failure in the `RULE path:line message` format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// The check rule label (C1..C7) or a command-level tag (e.g. `INIT`).
    pub rule: String,
    /// Repo-relative path of the offending file, or `.` for repo-level
    /// violations.
    pub path: String,
    /// 1-based line, or 0 when the violation is not attributable to a line.
    /// Formatting renders `:N` only when `line > 0`.
    pub line: u32,
    pub message: String,
}

impl Violation {
    pub fn new(rule: &str, path: impl Into<String>, line: u32, message: impl Into<String>) -> Self {
        Self {
            rule: rule.to_string(),
            path: path.into(),
            line,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line > 0 {
            write!(
                f,
                "{} {}:{} {}",
                self.rule, self.path, self.line, self.message
            )
        } else {
            write!(f, "{} {} {}", self.rule, self.path, self.message)
        }
    }
}

/// Command-level fatal error (not a per-rule violation): e.g. missing seed,
/// missing repo, I/O failure. These are printed plainly and exit 1.
#[derive(Debug)]
pub struct Fatal(pub String);

impl std::fmt::Display for Fatal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Fatal {
    fn from(s: &str) -> Self {
        Fatal(s.to_string())
    }
}

/// Find the 1-based line containing `needle` in `text`, else line 1.
///
/// Used to attribute JSON-path errors (C3/C4/C5) to a source line by
/// searching for the exact offending value in the YAML text.
pub fn line_of(text: &str, needle: &str) -> u32 {
    for (i, line) in text.lines().enumerate() {
        if line.contains(needle) {
            return i as u32 + 1;
        }
    }
    1
}
