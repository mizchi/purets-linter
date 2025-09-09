// Pure TypeScript Linter Library

use oxc::span::Span;
use std::path::{Path, PathBuf};
use crate::disable_directives::DisableDirectives;
use crate::expect_error_directives::ExpectErrorDirectives;

pub mod rules;
pub mod comparer;
pub mod combined_visitor;
pub mod package_checker;
pub mod disable_directives;
pub mod expect_error_directives;
pub mod test_runner;
pub mod test_runner_detector;
pub mod presets;
pub mod init;
pub mod workspace_detector;
pub mod gitignore_filter;
#[cfg(test)]
pub mod test_utils;
mod tsconfig_validator;
mod package_json_validator;

pub use tsconfig_validator::TsConfigValidator;
pub use package_json_validator::PackageJsonValidator;
pub use package_checker::check_package_json;
pub use test_runner::TestRunner;

pub struct Linter {
    pub path: PathBuf,
    pub source_text: String,
    pub errors: Vec<LintError>,
    pub verbose: bool,
    disable_directives: DisableDirectives,
    expect_error_directives: ExpectErrorDirectives,
    pub test_runner: Option<TestRunner>,
    pub is_entry_point: bool,
    pub is_main_entry: bool,
}

#[derive(Debug)]
pub struct LintError {
    pub rule: String,
    pub message: String,
    pub span: Span,
}

impl Linter {
    pub fn new(path: &Path, source_text: &str, verbose: bool) -> Self {
        let disable_directives = DisableDirectives::from_source(source_text);
        let expect_error_directives = ExpectErrorDirectives::from_source(source_text);
        
        Self {
            path: path.to_path_buf(),
            source_text: source_text.to_string(),
            errors: Vec::new(),
            verbose,
            disable_directives,
            expect_error_directives,
            test_runner: None,
            is_entry_point: false,
            is_main_entry: false,
        }
    }
    
    pub fn with_test_runner(mut self, test_runner: Option<TestRunner>) -> Self {
        self.test_runner = test_runner;
        self
    }
    
    pub fn with_entry_point(mut self, is_entry: bool) -> Self {
        self.is_entry_point = is_entry;
        self
    }
    
    pub fn with_main_entry(mut self, is_main: bool) -> Self {
        self.is_main_entry = is_main;
        self
    }
    
    pub fn check_program(&mut self, program: &oxc::ast::ast::Program) {
        // Use combined visitor for better performance
        use crate::combined_visitor::check_program_combined;
        check_program_combined(self, program);
    }
    
    pub fn add_error(&mut self, rule: String, message: String, span: Span) {
        // Get the line number from the span
        let (line, _) = self.get_position(span.start);
        
        // Check if this error should be disabled
        if self.disable_directives.is_rule_disabled(line - 1, &rule) {
            return; // Skip this error
        }
        
        // Check if this error is expected
        if self.expect_error_directives.is_error_expected(line - 1, &rule) {
            self.expect_error_directives.mark_as_triggered(line - 1, &rule);
            return; // Skip this error as it was expected
        }
        
        self.errors.push(LintError {
            rule,
            message,
            span,
        });
    }
    
    pub fn check_untriggered_expect_errors(&mut self) {
        // After all checks, report any untriggered expect-error directives
        let untriggered = self.expect_error_directives.get_untriggered_errors();
        
        for (line, rules) in untriggered {
            // Convert line number (0-based) to 1-based for display
            let display_line = line + 1;
            
            // Calculate span for the expect-error line
            let mut current_line = 0;
            let mut char_pos = 0;
            
            for ch in self.source_text.chars() {
                if current_line == line {
                    // Found the line, create a span for it
                    let span_start = char_pos;
                    // Find end of line
                    let mut span_end = char_pos;
                    for ch2 in self.source_text[char_pos..].chars() {
                        if ch2 == '\n' {
                            break;
                        }
                        span_end += ch2.len_utf8();
                    }
                    
                    let span = Span::new(span_start as u32, span_end as u32);
                    
                    for rule in rules {
                        self.errors.push(LintError {
                            rule: "unused-expect-error".to_string(),
                            message: format!(
                                "Expected error '{}' on line {} was not triggered",
                                rule, display_line
                            ),
                            span,
                        });
                    }
                    break;
                }
                
                if ch == '\n' {
                    current_line += 1;
                }
                char_pos += ch.len_utf8();
            }
        }
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn get_errors(&self) -> &[LintError] {
        &self.errors
    }
    
    pub fn report_errors(&self) {
        use colored::*;
        
        for error in &self.errors {
            let (line, column) = self.get_position(error.span.start);
            
            // VSCode-compatible format: file:line:column
            eprintln!(
                "{} {} {}",
                format!("{}:{}:{}", self.path.display(), line, column).cyan().bold(),
                format!("[{}]", error.rule).yellow(),
                error.message.white()
            );
            
            if self.verbose {
                if let Some(line_text) = self.get_line_text(line) {
                    eprintln!("  {}", line_text.dimmed());
                    eprintln!("  {}{}\n", 
                        " ".repeat(column - 1), 
                        "^".red().bold()
                    );
                }
            }
        }
    }
    
    fn get_position(&self, offset: u32) -> (usize, usize) {
        let mut line = 1;
        let mut column = 1;
        
        for (i, ch) in self.source_text.chars().enumerate() {
            if i as u32 >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        
        (line, column)
    }
    
    fn get_line_text(&self, line_number: usize) -> Option<String> {
        self.source_text
            .lines()
            .nth(line_number - 1)
            .map(|s| s.to_string())
    }
}