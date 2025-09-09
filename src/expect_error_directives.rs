use std::collections::HashMap;
use std::cell::RefCell;

/// Manages purets-expect-error directives
#[derive(Debug, Default)]
pub struct ExpectErrorDirectives {
    /// Maps line numbers to expected error rules
    expected_errors: HashMap<usize, Vec<String>>,
    /// Tracks which expected errors were actually triggered
    triggered_errors: RefCell<HashMap<usize, Vec<String>>>,
}

impl ExpectErrorDirectives {
    /// Parse expect-error directives from source code
    pub fn from_source(source: &str) -> Self {
        let mut expected_errors = HashMap::new();
        
        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            
            // Check for purets-expect-error comment
            if let Some(comment_start) = trimmed.find("// purets-expect-error") {
                let comment = &trimmed[comment_start..];
                
                // Extract rule names after the directive
                if let Some(rules_start) = comment.find("purets-expect-error") {
                    let rules_part = &comment[rules_start + "purets-expect-error".len()..];
                    let rules_part = rules_part.trim();
                    
                    if !rules_part.is_empty() {
                        // Split by comma or whitespace and collect rule names
                        let rules: Vec<String> = rules_part
                            .split(|c: char| c == ',' || c.is_whitespace())
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .map(String::from)
                            .collect();
                        
                        if !rules.is_empty() {
                            // The error is expected on the NEXT line
                            expected_errors.insert(line_num + 1, rules);
                        }
                    }
                }
            }
        }
        
        Self {
            expected_errors,
            triggered_errors: RefCell::new(HashMap::new()),
        }
    }
    
    /// Check if an error is expected at the given line
    pub fn is_error_expected(&self, line: usize, rule: &str) -> bool {
        if let Some(expected_rules) = self.expected_errors.get(&line) {
            expected_rules.iter().any(|r| r == rule)
        } else {
            false
        }
    }
    
    /// Mark an expected error as triggered
    pub fn mark_as_triggered(&self, line: usize, rule: &str) {
        self.triggered_errors
            .borrow_mut()
            .entry(line)
            .or_insert_with(Vec::new)
            .push(rule.to_string());
    }
    
    /// Get all untriggered expected errors
    pub fn get_untriggered_errors(&self) -> Vec<(usize, Vec<String>)> {
        let mut untriggered = Vec::new();
        let triggered_errors = self.triggered_errors.borrow();
        
        for (line, expected_rules) in &self.expected_errors {
            let triggered = triggered_errors.get(line);
            
            let untriggered_rules: Vec<String> = expected_rules
                .iter()
                .filter(|rule| {
                    if let Some(triggered_rules) = triggered {
                        !triggered_rules.contains(rule)
                    } else {
                        true
                    }
                })
                .cloned()
                .collect();
            
            if !untriggered_rules.is_empty() {
                untriggered.push((*line, untriggered_rules));
            }
        }
        
        untriggered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_expect_error() {
        let source = r#"
function test() {
    // purets-expect-error no-console
    console.log("test");
    
    // purets-expect-error no-any no-explicit-any
    const x: any = 123;
}
"#;
        
        let directives = ExpectErrorDirectives::from_source(source);
        
        // Line 3 (console.log) should expect no-console
        assert!(directives.is_error_expected(3, "no-console"));
        assert!(!directives.is_error_expected(3, "no-any"));
        
        // Line 6 (const x: any) should expect both no-any and no-explicit-any
        assert!(directives.is_error_expected(6, "no-any"));
        assert!(directives.is_error_expected(6, "no-explicit-any"));
        
        // Other lines should not expect errors
        assert!(!directives.is_error_expected(1, "no-console"));
        assert!(!directives.is_error_expected(4, "no-console"));
    }
    
    #[test]
    fn test_untriggered_errors() {
        let source = r#"
// purets-expect-error no-console
console.log("test");

// purets-expect-error no-any no-explicit-any
const x: any = 123;
"#;
        
        let directives = ExpectErrorDirectives::from_source(source);
        
        // Mark only no-console as triggered
        directives.mark_as_triggered(2, "no-console");
        directives.mark_as_triggered(5, "no-any");
        
        let untriggered = directives.get_untriggered_errors();
        
        // Should have one untriggered error (no-explicit-any on line 5)
        assert_eq!(untriggered.len(), 1);
        assert_eq!(untriggered[0].0, 5);
        assert_eq!(untriggered[0].1, vec!["no-explicit-any".to_string()]);
    }
    
    #[test]
    fn test_comma_separated_rules() {
        let source = r#"
// purets-expect-error no-console, no-any, no-explicit-any
const x: any = console.log("test");
"#;
        
        let directives = ExpectErrorDirectives::from_source(source);
        
        assert!(directives.is_error_expected(2, "no-console"));
        assert!(directives.is_error_expected(2, "no-any"));
        assert!(directives.is_error_expected(2, "no-explicit-any"));
    }
}