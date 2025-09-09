use std::fmt;

/// Test runner configuration
#[derive(Debug, Clone, PartialEq)]
pub enum TestRunner {
    Vitest,
    NodeTest,
    DenoTest,
}

impl TestRunner {
    /// Parse test runner from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "vitest" => Some(TestRunner::Vitest),
            "node-test" => Some(TestRunner::NodeTest),
            "deno-test" => Some(TestRunner::DenoTest),
            _ => None,
        }
    }

    /// Get the expected import patterns for this test runner
    pub fn get_import_patterns(&self) -> Vec<&'static str> {
        match self {
            TestRunner::Vitest => vec!["vitest", "@vitest/"],
            TestRunner::NodeTest => vec!["node:test", "node:assert"],
            TestRunner::DenoTest => vec![
                "deno.land/std/testing",
                "deno.land/std/assert",
                "@std/expect",
                "@std/assert",
                "jsr:@std/expect",
                "jsr:@std/assert",
            ],
        }
    }

    /// Check if an import source matches this test runner
    pub fn matches_import(&self, import_source: &str) -> bool {
        self.get_import_patterns()
            .iter()
            .any(|pattern| import_source.contains(pattern))
    }

    /// Get the test function names for this runner
    pub fn get_test_functions(&self) -> Vec<&'static str> {
        match self {
            TestRunner::Vitest => vec!["describe", "it", "test", "beforeEach", "afterEach"],
            TestRunner::NodeTest => vec!["describe", "it", "test", "before", "after"],
            TestRunner::DenoTest => vec!["Deno.test"],
        }
    }
}

impl fmt::Display for TestRunner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestRunner::Vitest => write!(f, "vitest"),
            TestRunner::NodeTest => write!(f, "node-test"),
            TestRunner::DenoTest => write!(f, "deno-test"),
        }
    }
}
