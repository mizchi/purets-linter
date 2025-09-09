use serde_json::Value;
use std::fs;
use std::path::Path;

// Forbidden libraries
const FORBIDDEN_LIBRARIES: &[&str] = &[
    "jquery",
    "lodash",
    "lodash.debounce",
    "lodash.throttle", 
    "lodash.merge",
    "lodash-es",
    "underscore",
    "rxjs",
];

// Libraries with better alternatives
const PREFER_ALTERNATIVES: &[(&str, &str)] = &[
    ("minimist", "node:util parseArgs"),
    ("yargs", "node:util parseArgs"),
    ("yargs-parser", "node:util parseArgs"),
    ("commander", "node:util parseArgs"),
    ("meow", "node:util parseArgs"),
];

pub fn check_package_json(project_path: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let package_json_path = project_path.join("package.json");
    
    if !package_json_path.exists() {
        return errors;
    }
    
    let contents = match fs::read_to_string(&package_json_path) {
        Ok(c) => c,
        Err(_) => return errors,
    };
    
    let json: Value = match serde_json::from_str(&contents) {
        Ok(j) => j,
        Err(e) => {
            errors.push(format!("Failed to parse package.json: {}", e));
            return errors;
        }
    };
    
    // Check dependencies
    check_dependencies(&json, "dependencies", &mut errors);
    check_dependencies(&json, "devDependencies", &mut errors);
    check_dependencies(&json, "peerDependencies", &mut errors);
    check_dependencies(&json, "optionalDependencies", &mut errors);
    
    errors
}

fn check_dependencies(json: &Value, field: &str, errors: &mut Vec<String>) {
    if let Some(deps) = json.get(field).and_then(|v| v.as_object()) {
        for (name, _version) in deps {
            // Check forbidden libraries
            if FORBIDDEN_LIBRARIES.contains(&name.as_str()) || name.starts_with("lodash.") {
                errors.push(format!(
                    "[package.json] Forbidden library '{}' found in {}. Consider using modern alternatives",
                    name, field
                ));
            }
            
            // Check libraries with alternatives
            for (lib, alternative) in PREFER_ALTERNATIVES {
                if name == lib {
                    errors.push(format!(
                        "[package.json] Library '{}' in {} has a better alternative. Use '{}' instead",
                        name, field, alternative
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_forbidden_libraries_in_package_json() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = r#"{
            "name": "test-project",
            "dependencies": {
                "jquery": "^3.6.0",
                "lodash": "^4.17.21",
                "express": "^4.18.0"
            },
            "devDependencies": {
                "underscore": "^1.13.0",
                "rxjs": "^7.5.0"
            }
        }"#;
        
        fs::write(temp_dir.path().join("package.json"), package_json).unwrap();
        
        let errors = check_package_json(temp_dir.path());
        assert_eq!(errors.len(), 4);
        assert!(errors.iter().any(|e| e.contains("jquery")));
        assert!(errors.iter().any(|e| e.contains("lodash")));
        assert!(errors.iter().any(|e| e.contains("underscore")));
        assert!(errors.iter().any(|e| e.contains("rxjs")));
    }

    #[test]
    fn test_alternatives_in_package_json() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = r#"{
            "name": "test-project",
            "dependencies": {
                "minimist": "^1.2.5",
                "yargs": "^17.0.0"
            }
        }"#;
        
        fs::write(temp_dir.path().join("package.json"), package_json).unwrap();
        
        let errors = check_package_json(temp_dir.path());
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("minimist"));
        assert!(errors[0].contains("node:util parseArgs"));
        assert!(errors[1].contains("yargs"));
    }

    #[test]
    fn test_lodash_variants() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = r#"{
            "name": "test-project",
            "dependencies": {
                "lodash.debounce": "^4.0.8",
                "lodash.merge": "^4.6.2",
                "lodash-es": "^4.17.21"
            }
        }"#;
        
        fs::write(temp_dir.path().join("package.json"), package_json).unwrap();
        
        let errors = check_package_json(temp_dir.path());
        assert_eq!(errors.len(), 3);
        assert!(errors.iter().any(|e| e.contains("lodash.debounce")));
        assert!(errors.iter().any(|e| e.contains("lodash.merge")));
        assert!(errors.iter().any(|e| e.contains("lodash-es")));
    }

    #[test]
    fn test_allowed_libraries() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = r#"{
            "name": "test-project",
            "dependencies": {
                "react": "^18.0.0",
                "express": "^4.18.0",
                "axios": "^1.0.0"
            }
        }"#;
        
        fs::write(temp_dir.path().join("package.json"), package_json).unwrap();
        
        let errors = check_package_json(temp_dir.path());
        assert_eq!(errors.len(), 0);
    }
}