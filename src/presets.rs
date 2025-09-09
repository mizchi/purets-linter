use std::collections::HashSet;

/// Rule presets based on naming conventions
#[derive(Debug, Clone)]
pub struct RulePreset {
    pub name: String,
    pub description: String,
    pub enabled_rules: HashSet<String>,
    pub disabled_rules: HashSet<String>,
}

impl RulePreset {
    /// Check if a rule is enabled
    pub fn is_rule_enabled(&self, rule_name: &str) -> Option<bool> {
        if self.disabled_rules.contains(rule_name) {
            Some(false)
        } else if self.enabled_rules.contains(rule_name) {
            Some(true)
        } else {
            None
        }
    }
    
    /// Strict preset - all rules enabled
    pub fn strict() -> Self {
        Self {
            name: "strict".to_string(),
            description: "All rules enabled for maximum strictness".to_string(),
            enabled_rules: HashSet::from([
                // Basic restrictions
                "no-classes".to_string(),
                "no-enums".to_string(),
                "no-throw".to_string(),
                "no-delete".to_string(),
                "no-eval-function".to_string(),
                "no-foreach".to_string(),
                "no-do-while".to_string(),
                // Type safety
                "no-as-cast".to_string(),
                "let-requires-type".to_string(),
                "empty-array-requires-type".to_string(),
                "prefer-readonly-array".to_string(),
                "no-mutable-record".to_string(),
                // Code quality
                "no-unused-variables".to_string(),
                "no-unused-map".to_string(),
                "must-use-return-value".to_string(),
                "catch-error-handling".to_string(),
                "switch-case-block".to_string(),
                // Import/Export
                "strict-named-export".to_string(),
                "no-namespace-imports".to_string(),
                "no-reexports".to_string(),
                "import-extensions".to_string(),
                "no-http-imports".to_string(),
                // Node.js compatibility
                "no-require".to_string(),
                "no-filename-dirname".to_string(),
                "no-global-process".to_string(),
                "node-import-style".to_string(),
                "forbidden-libraries".to_string(),
                // Function restrictions
                "max-function-params".to_string(),
                "no-this-in-functions".to_string(),
                "no-side-effect-functions".to_string(),
                "filename-function-match".to_string(),
                "export-requires-jsdoc".to_string(),
                "jsdoc-param-match".to_string(),
                // Path-based restrictions
                "path-based-restrictions".to_string(),
                // Side effects
                "no-top-level-side-effects".to_string(),
            ]),
            disabled_rules: HashSet::new(),
        }
    }

    /// Relaxed preset - for existing codebases migrating to pure-ts
    pub fn relaxed() -> Self {
        Self {
            name: "relaxed".to_string(),
            description: "Relaxed rules for gradual migration".to_string(),
            enabled_rules: HashSet::from([
                // Only critical rules
                "no-eval-function".to_string(),
                "no-delete".to_string(),
                "no-unused-variables".to_string(),
                "catch-error-handling".to_string(),
                "no-http-imports".to_string(),
                "forbidden-libraries".to_string(),
            ]),
            disabled_rules: HashSet::from([
                // Allow these for easier migration
                "no-classes".to_string(),
                "no-throw".to_string(),
                "strict-named-export".to_string(),
                "filename-function-match".to_string(),
                "export-requires-jsdoc".to_string(),
                "no-top-level-side-effects".to_string(),
            ]),
        }
    }

    /// Functional preset - enforces functional programming style
    pub fn functional() -> Self {
        Self {
            name: "functional".to_string(),
            description: "Functional programming style enforcement".to_string(),
            enabled_rules: HashSet::from([
                // Core FP rules
                "no-classes".to_string(),
                "no-this-in-functions".to_string(),
                "no-foreach".to_string(),
                "no-do-while".to_string(),
                "no-delete".to_string(),
                "no-member-assignments".to_string(),
                "no-object-assign".to_string(),
                "prefer-readonly-array".to_string(),
                "no-mutable-record".to_string(),
                // Pure functions
                "no-side-effect-functions".to_string(),
                "path-based-restrictions".to_string(),
                // Immutability
                "let-requires-type".to_string(),
                "empty-array-requires-type".to_string(),
            ]),
            disabled_rules: HashSet::from([
                // Allow some OO patterns
                "strict-named-export".to_string(),
                "filename-function-match".to_string(),
            ]),
        }
    }

    /// Library preset - for library/package development
    pub fn library() -> Self {
        Self {
            name: "library".to_string(),
            description: "Rules optimized for library development".to_string(),
            enabled_rules: HashSet::from([
                // Quality and documentation
                "export-requires-jsdoc".to_string(),
                "jsdoc-param-match".to_string(),
                "no-unused-variables".to_string(),
                "must-use-return-value".to_string(),
                // Type safety
                "no-as-cast".to_string(),
                "let-requires-type".to_string(),
                "prefer-readonly-array".to_string(),
                // Clean exports
                "no-reexports".to_string(),
                "filename-function-match".to_string(),
                // No side effects
                "no-top-level-side-effects".to_string(),
                "no-side-effect-functions".to_string(),
            ]),
            disabled_rules: HashSet::from([
                // Allow flexible patterns for library APIs
                "no-classes".to_string(),
                "strict-named-export".to_string(),
                "max-function-params".to_string(),
            ]),
        }
    }

    /// Test preset - for test files
    pub fn test() -> Self {
        Self {
            name: "test".to_string(),
            description: "Rules for test files".to_string(),
            enabled_rules: HashSet::from([
                // Basic quality
                "no-unused-variables".to_string(),
                "catch-error-handling".to_string(),
                "import-extensions".to_string(),
            ]),
            disabled_rules: HashSet::from([
                // Allow test patterns
                "no-top-level-side-effects".to_string(),
                "filename-function-match".to_string(),
                "export-requires-jsdoc".to_string(),
                "no-throw".to_string(),
                "max-function-params".to_string(),
            ]),
        }
    }

    /// Get preset by name
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "strict" => Some(Self::strict()),
            "relaxed" => Some(Self::relaxed()),
            "functional" => Some(Self::functional()),
            "library" => Some(Self::library()),
            "test" => Some(Self::test()),
            _ => None,
        }
    }
}

/// Get strict preset for config
pub fn get_strict_preset() -> Preset {
    Preset {
        rules: RulePreset::strict()
            .enabled_rules
            .into_iter()
            .map(|r| (r, true))
            .collect(),
    }
}

/// Get relaxed preset for config
pub fn get_relaxed_preset() -> Preset {
    Preset {
        rules: RulePreset::relaxed()
            .enabled_rules
            .into_iter()
            .map(|r| (r, true))
            .collect(),
    }
}

/// Get recommended preset for config (defaults to functional)
pub fn get_recommended_preset() -> Preset {
    Preset {
        rules: RulePreset::functional()
            .enabled_rules
            .into_iter()
            .map(|r| (r, true))
            .collect(),
    }
}

/// Simple preset structure for config
pub struct Preset {
    pub rules: std::collections::HashMap<String, bool>,
}

impl Preset {
    /// List all available presets
    pub fn list_all() -> Vec<&'static str> {
        vec!["strict", "relaxed", "functional", "library", "test"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_preset() {
        let preset = RulePreset::strict();
        assert!(preset.is_rule_enabled("no-classes").unwrap());
        assert!(preset.is_rule_enabled("no-throw").unwrap());
    }

    #[test]
    fn test_relaxed_preset() {
        let preset = RulePreset::relaxed();
        assert_eq!(preset.is_rule_enabled("no-classes"), Some(false));
        assert!(preset.is_rule_enabled("no-eval-function").unwrap());
    }

    #[test]
    fn test_preset_from_name() {
        assert!(RulePreset::from_name("strict").is_some());
        assert!(RulePreset::from_name("invalid").is_none());
    }
}
