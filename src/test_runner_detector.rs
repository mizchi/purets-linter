use std::fs;
use std::path::{Path, PathBuf};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum TestRunner {
    Vitest,
    NodeTest,
    DenoTest,
    None,
}

impl TestRunner {
    pub fn as_str(&self) -> &str {
        match self {
            TestRunner::Vitest => "vitest",
            TestRunner::NodeTest => "node-test",
            TestRunner::DenoTest => "deno-test",
            TestRunner::None => "none",
        }
    }
}

pub struct TestRunnerDetector {
    root: PathBuf,
}

impl TestRunnerDetector {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn detect(&self) -> TestRunner {
        // Check for Deno first (deno.json or deno.jsonc)
        if self.has_deno_config() {
            return TestRunner::DenoTest;
        }

        // Check for Vitest in package.json dependencies
        if self.has_vitest_dependency() {
            return TestRunner::Vitest;
        }

        // Check for Node.js test imports in source files
        if self.has_node_test_imports() {
            return TestRunner::NodeTest;
        }

        TestRunner::None
    }

    fn has_deno_config(&self) -> bool {
        let deno_json = self.root.join("deno.json");
        let deno_jsonc = self.root.join("deno.jsonc");
        deno_json.exists() || deno_jsonc.exists()
    }

    fn has_vitest_dependency(&self) -> bool {
        let package_json_path = self.root.join("package.json");
        if !package_json_path.exists() {
            return false;
        }

        if let Ok(content) = fs::read_to_string(&package_json_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                // Check devDependencies
                if let Some(dev_deps) = json.get("devDependencies") {
                    if dev_deps.get("vitest").is_some() {
                        return true;
                    }
                }
                // Also check regular dependencies (less common but possible)
                if let Some(deps) = json.get("dependencies") {
                    if deps.get("vitest").is_some() {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn has_node_test_imports(&self) -> bool {
        // Check common test directories
        let test_dirs = vec![
            self.root.join("test"),
            self.root.join("tests"),
            self.root.join("src"),
            self.root.join("__tests__"),
        ];

        for dir in test_dirs {
            if dir.exists() {
                if self.check_directory_for_node_test(&dir) {
                    return true;
                }
            }
        }

        false
    }

    fn check_directory_for_node_test(&self, dir: &Path) -> bool {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Check TypeScript and JavaScript test files
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "ts" || ext == "js" || ext == "mjs" || ext == "tsx" || ext == "jsx" {
                            if let Some(name) = path.file_name() {
                                let name_str = name.to_string_lossy();
                                // Check if it's a test file (Vitest pattern: .test.ts, .spec.ts)
                                if name_str.contains(".test.") || name_str.contains(".spec.") || 
                                   name_str.contains("_test.") || name_str.contains("_spec.") {
                                    if self.file_has_node_test_import(&path) {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                } else if path.is_dir() && !path.ends_with("node_modules") {
                    // Recursively check subdirectories
                    if self.check_directory_for_node_test(&path) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn file_has_node_test_import(&self, file_path: &Path) -> bool {
        if let Ok(content) = fs::read_to_string(file_path) {
            // Check for various forms of node:test import
            content.contains("from \"node:test\"") ||
            content.contains("from 'node:test'") ||
            content.contains("require(\"node:test\")") ||
            content.contains("require('node:test')") ||
            content.contains("from\"node:test\"") ||
            content.contains("from'node:test'")
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_vitest() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = r#"{
            "devDependencies": {
                "vitest": "^1.0.0"
            }
        }"#;
        fs::write(temp_dir.path().join("package.json"), package_json).unwrap();

        let detector = TestRunnerDetector::new(temp_dir.path().to_path_buf());
        assert_eq!(detector.detect(), TestRunner::Vitest);
    }

    #[test]
    fn test_detect_deno() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("deno.json"), "{}").unwrap();

        let detector = TestRunnerDetector::new(temp_dir.path().to_path_buf());
        assert_eq!(detector.detect(), TestRunner::DenoTest);
    }

    #[test]
    fn test_detect_node_test() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("test");
        fs::create_dir(&test_dir).unwrap();
        
        let test_file = r#"
import { test } from "node:test";
import assert from "node:assert";

test("example", () => {
    assert.equal(1, 1);
});
"#;
        fs::write(test_dir.join("example.test.ts"), test_file).unwrap();

        let detector = TestRunnerDetector::new(temp_dir.path().to_path_buf());
        assert_eq!(detector.detect(), TestRunner::NodeTest);
    }

    #[test]
    fn test_detect_none() {
        let temp_dir = TempDir::new().unwrap();
        let detector = TestRunnerDetector::new(temp_dir.path().to_path_buf());
        assert_eq!(detector.detect(), TestRunner::None);
    }
}