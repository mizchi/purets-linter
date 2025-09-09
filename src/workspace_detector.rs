use glob::glob;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// Workspace configuration
#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    pub workspace_type: WorkspaceType,
    pub packages: Vec<String>,
    pub root: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceType {
    Single,
    NpmWorkspaces,
    PnpmWorkspaces,
    YarnWorkspaces,
}

impl WorkspaceConfig {
    /// Detect workspace configuration from current directory
    pub fn detect(root: &Path) -> Self {
        // Check for pnpm-workspace.yaml
        if let Ok(config) = detect_pnpm_workspace(root) {
            return config;
        }

        // Check for package.json workspaces
        if let Ok(config) = detect_npm_workspace(root) {
            return config;
        }

        // Default to single package
        Self {
            workspace_type: WorkspaceType::Single,
            packages: vec!["src".to_string()],
            root: root.to_path_buf(),
        }
    }

    /// Get all target directories to check
    pub fn get_target_dirs(&self) -> Vec<PathBuf> {
        let mut targets = Vec::new();

        match self.workspace_type {
            WorkspaceType::Single => {
                // Single package - check src directory
                let src_dir = self.root.join("src");
                if src_dir.exists() {
                    targets.push(src_dir);
                }
            }
            _ => {
                // Monorepo - expand glob patterns
                for pattern in &self.packages {
                    let full_pattern = self.root.join(pattern).to_string_lossy().to_string();

                    // Try to expand glob pattern
                    if let Ok(paths) = glob(&full_pattern) {
                        for path in paths.flatten() {
                            // Check for src directory in each package
                            let src_dir = path.join("src");
                            if src_dir.exists() && src_dir.is_dir() {
                                targets.push(src_dir);
                            }

                            // Also check the package root if it contains TypeScript files
                            if path.is_dir() && has_typescript_files(&path) {
                                targets.push(path);
                            }
                        }
                    }
                }
            }
        }

        // If no targets found, default to current directory
        if targets.is_empty() {
            targets.push(self.root.clone());
        }

        // Remove duplicates and sort
        targets.sort();
        targets.dedup();

        targets
    }

    /// Check if running in monorepo context
    pub fn is_monorepo(&self) -> bool {
        self.workspace_type != WorkspaceType::Single
    }

    /// Get package name from path
    pub fn get_package_name(&self, path: &Path) -> Option<String> {
        if !self.is_monorepo() {
            return None;
        }

        // Try to extract package name from path
        let relative = path.strip_prefix(&self.root).ok()?;
        let components: Vec<_> = relative.components().collect();

        if components.len() >= 2 {
            // Expect structure like packages/package-name/src
            let package_type = components[0].as_os_str().to_str()?;
            let package_name = components[1].as_os_str().to_str()?;
            Some(format!("{}/{}", package_type, package_name))
        } else {
            None
        }
    }
}

/// Detect pnpm workspace configuration
fn detect_pnpm_workspace(root: &Path) -> Result<WorkspaceConfig, Box<dyn std::error::Error>> {
    let workspace_file = root.join("pnpm-workspace.yaml");
    if !workspace_file.exists() {
        return Err("No pnpm-workspace.yaml found".into());
    }

    let content = fs::read_to_string(&workspace_file)?;

    // Simple YAML parsing for packages field
    let mut packages = Vec::new();
    let mut in_packages = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("packages:") {
            in_packages = true;
            continue;
        }

        if in_packages {
            if trimmed.starts_with("- ") {
                let package = trimmed
                    .trim_start_matches("- ")
                    .trim_matches('\'')
                    .trim_matches('"')
                    .to_string();
                packages.push(package);
            } else if !trimmed.starts_with('#') && !trimmed.is_empty() {
                // End of packages section
                break;
            }
        }
    }

    Ok(WorkspaceConfig {
        workspace_type: WorkspaceType::PnpmWorkspaces,
        packages,
        root: root.to_path_buf(),
    })
}

/// Detect npm/yarn workspace configuration from package.json
fn detect_npm_workspace(root: &Path) -> Result<WorkspaceConfig, Box<dyn std::error::Error>> {
    let package_json = root.join("package.json");
    if !package_json.exists() {
        return Err("No package.json found".into());
    }

    let content = fs::read_to_string(&package_json)?;
    let json: Value = serde_json::from_str(&content)?;

    // Check for workspaces field
    if let Some(workspaces) = json.get("workspaces") {
        let packages = if workspaces.is_array() {
            // Yarn classic / npm workspaces as array
            workspaces
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        } else if let Some(packages_field) = workspaces.get("packages") {
            // Yarn 2+ with packages field
            packages_field
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        } else {
            Vec::new()
        };

        if !packages.is_empty() {
            // Detect if using Yarn by checking for yarn.lock
            let workspace_type = if root.join("yarn.lock").exists() {
                WorkspaceType::YarnWorkspaces
            } else {
                WorkspaceType::NpmWorkspaces
            };

            return Ok(WorkspaceConfig {
                workspace_type,
                packages,
                root: root.to_path_buf(),
            });
        }
    }

    Err("No workspaces configuration found".into())
}

/// Check if directory contains TypeScript files
fn has_typescript_files(dir: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ext == "ts" || ext == "tsx" || ext == "mts" || ext == "cts" {
                    return true;
                }
            }
        }
    }
    false
}

/// Common monorepo patterns
pub fn get_common_workspace_patterns() -> Vec<String> {
    vec![
        "packages/*".to_string(),
        "apps/*".to_string(),
        "services/*".to_string(),
        "libs/*".to_string(),
        "tools/*".to_string(),
        "examples/*".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_single_package_detection() {
        let temp_dir = TempDir::new().unwrap();
        let config = WorkspaceConfig::detect(temp_dir.path());

        assert_eq!(config.workspace_type, WorkspaceType::Single);
        assert_eq!(config.packages, vec!["src"]);
    }

    #[test]
    fn test_pnpm_workspace_detection() {
        let temp_dir = TempDir::new().unwrap();

        // Create pnpm-workspace.yaml
        let workspace_content = r#"
packages:
  - 'packages/*'
  - 'apps/*'
  - '!**/test/**'
"#;
        fs::write(
            temp_dir.path().join("pnpm-workspace.yaml"),
            workspace_content,
        )
        .unwrap();

        let config = WorkspaceConfig::detect(temp_dir.path());

        assert_eq!(config.workspace_type, WorkspaceType::PnpmWorkspaces);
        assert_eq!(config.packages.len(), 3);
        assert!(config.packages.contains(&"packages/*".to_string()));
        assert!(config.packages.contains(&"apps/*".to_string()));
    }

    #[test]
    fn test_npm_workspace_detection() {
        let temp_dir = TempDir::new().unwrap();

        // Create package.json with workspaces
        let package_json = r#"{
            "name": "monorepo",
            "workspaces": [
                "packages/*",
                "apps/*"
            ]
        }"#;
        fs::write(temp_dir.path().join("package.json"), package_json).unwrap();

        let config = WorkspaceConfig::detect(temp_dir.path());

        assert_eq!(config.workspace_type, WorkspaceType::NpmWorkspaces);
        assert_eq!(config.packages.len(), 2);
    }

    #[test]
    fn test_yarn_workspace_detection() {
        let temp_dir = TempDir::new().unwrap();

        // Create package.json with workspaces
        let package_json = r#"{
            "name": "monorepo",
            "workspaces": {
                "packages": [
                    "packages/*",
                    "apps/*"
                ]
            }
        }"#;
        fs::write(temp_dir.path().join("package.json"), package_json).unwrap();

        // Create yarn.lock to indicate Yarn
        fs::write(temp_dir.path().join("yarn.lock"), "").unwrap();

        let config = WorkspaceConfig::detect(temp_dir.path());

        assert_eq!(config.workspace_type, WorkspaceType::YarnWorkspaces);
    }

    #[test]
    fn test_get_target_dirs() {
        let temp_dir = TempDir::new().unwrap();

        // Create directory structure
        fs::create_dir_all(temp_dir.path().join("packages/pkg1/src")).unwrap();
        fs::create_dir_all(temp_dir.path().join("packages/pkg2/src")).unwrap();
        fs::create_dir_all(temp_dir.path().join("apps/app1/src")).unwrap();

        // Create pnpm-workspace.yaml
        let workspace_content = r#"
packages:
  - 'packages/*'
  - 'apps/*'
"#;
        fs::write(
            temp_dir.path().join("pnpm-workspace.yaml"),
            workspace_content,
        )
        .unwrap();

        let config = WorkspaceConfig::detect(temp_dir.path());
        let targets = config.get_target_dirs();

        assert!(targets.len() >= 3);
        assert!(targets.iter().any(|p| p.ends_with("packages/pkg1/src")));
        assert!(targets.iter().any(|p| p.ends_with("packages/pkg2/src")));
        assert!(targets.iter().any(|p| p.ends_with("apps/app1/src")));
    }
}
