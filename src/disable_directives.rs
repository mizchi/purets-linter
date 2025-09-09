use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct DisableDirectives {
    /// Lines that should be ignored (0-indexed)
    pub disabled_lines: HashSet<usize>,
    /// Whether the entire file is disabled
    pub file_disabled: bool,
    /// Specific rules disabled for lines (line_number -> set of rule names)
    pub line_rule_overrides: std::collections::HashMap<usize, HashSet<String>>,
}

impl DisableDirectives {
    pub fn from_source(source_text: &str) -> Self {
        let mut directives = Self::default();

        for (line_idx, line) in source_text.lines().enumerate() {
            let trimmed = line.trim();

            // Check for file-level disable
            if trimmed.contains("// purets-disable-file")
                || trimmed.contains("/* purets-disable-file")
            {
                directives.file_disabled = true;
                continue;
            }

            // Check for next-line disable
            if trimmed.contains("// purets-disable-next-line") {
                // Disable the next line (current line + 1)
                directives.disabled_lines.insert(line_idx + 1);

                // Check for specific rules after the directive
                if let Some(rules_start) = trimmed.find("// purets-disable-next-line") {
                    let after_directive =
                        &trimmed[rules_start + "// purets-disable-next-line".len()..];
                    let rules = parse_rule_names(after_directive);
                    if !rules.is_empty() {
                        directives
                            .line_rule_overrides
                            .entry(line_idx + 1)
                            .or_insert_with(HashSet::new)
                            .extend(rules);
                    }
                }
            }

            // Also check for inline disable on the same line
            if trimmed.contains("// purets-disable-line") {
                directives.disabled_lines.insert(line_idx);

                // Check for specific rules
                if let Some(rules_start) = trimmed.find("// purets-disable-line") {
                    let after_directive = &trimmed[rules_start + "// purets-disable-line".len()..];
                    let rules = parse_rule_names(after_directive);
                    if !rules.is_empty() {
                        directives
                            .line_rule_overrides
                            .entry(line_idx)
                            .or_insert_with(HashSet::new)
                            .extend(rules);
                    }
                }
            }
        }

        directives
    }

    /// Check if a specific line is disabled
    pub fn is_line_disabled(&self, line: usize) -> bool {
        self.file_disabled || self.disabled_lines.contains(&line)
    }

    /// Check if a specific rule is disabled for a line
    pub fn is_rule_disabled(&self, line: usize, rule: &str) -> bool {
        if self.file_disabled {
            return true;
        }

        if self.disabled_lines.contains(&line) {
            // If no specific rules are specified, all are disabled
            if let Some(rules) = self.line_rule_overrides.get(&line) {
                return rules.is_empty() || rules.contains(rule);
            }
            return true;
        }

        // Check if this specific rule is disabled
        if let Some(rules) = self.line_rule_overrides.get(&line) {
            return rules.contains(rule);
        }

        false
    }
}

fn parse_rule_names(text: &str) -> Vec<String> {
    // Parse rule names from text like: "no-console, no-eval"
    text.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !s.starts_with("*/"))
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disable_next_line() {
        let source = r#"
// purets-disable-next-line
console.log("This line is disabled");
console.log("This line is not disabled");
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_line_disabled(2)); // Line after comment is disabled
        assert!(!directives.is_line_disabled(3)); // Next line is not
    }

    #[test]
    fn test_disable_file() {
        let source = r#"
// purets-disable-file
console.log("Everything is disabled");
console.log("This too");
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.file_disabled);
        assert!(directives.is_line_disabled(2));
        assert!(directives.is_line_disabled(3));
    }

    #[test]
    fn test_disable_specific_rules() {
        let source = r#"
// purets-disable-next-line no-console, allow-directives
console.log("console disabled");
// purets-disable-next-line no-eval
eval("code");
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_rule_disabled(2, "no-console"));
        assert!(directives.is_rule_disabled(2, "allow-directives"));
        assert!(!directives.is_rule_disabled(2, "no-eval"));

        assert!(directives.is_rule_disabled(4, "no-eval"));
        assert!(!directives.is_rule_disabled(4, "no-console"));
    }

    #[test]
    fn test_disable_line() {
        let source = r#"
console.log("test"); // purets-disable-line
eval("code"); // purets-disable-line no-eval
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_line_disabled(1)); // First console.log line
        assert!(directives.is_rule_disabled(2, "no-eval")); // eval line with specific rule
    }

    #[test]
    fn test_multiple_rules_disable() {
        let source = r#"
// purets-disable-next-line no-console, allow-directives, no-eval
console.log(eval("test"));
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_rule_disabled(2, "no-console"));
        assert!(directives.is_rule_disabled(2, "allow-directives"));
        assert!(directives.is_rule_disabled(2, "no-eval"));
        assert!(!directives.is_rule_disabled(2, "other-rule"));
    }

    #[test]
    fn test_disable_affects_correct_line() {
        let source = r#"
console.log("line 1");
// purets-disable-next-line
console.log("line 3");
console.log("line 4");
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(!directives.is_line_disabled(1)); // Line 1 not disabled
        assert!(directives.is_line_disabled(3)); // Line 3 is disabled
        assert!(!directives.is_line_disabled(4)); // Line 4 not disabled
    }

    #[test]
    fn test_file_disable_overrides_all() {
        let source = r#"
// purets-disable-file
// purets-disable-next-line no-console
console.log("test");
document.body;
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.file_disabled);
        assert!(directives.is_line_disabled(3)); // All lines disabled
        assert!(directives.is_line_disabled(4)); // All lines disabled
        assert!(directives.is_rule_disabled(3, "any-rule")); // Any rule disabled
    }

    #[test]
    fn test_inline_disable_same_line() {
        let source = r#"
const x = eval("code"); // purets-disable-line no-eval
const y = eval("code"); // Not disabled
"#;
        let directives = DisableDirectives::from_source(source);

        assert!(directives.is_rule_disabled(1, "no-eval"));
        assert!(!directives.is_rule_disabled(1, "other-rule"));
        assert!(!directives.is_rule_disabled(2, "no-eval"));
    }
}
