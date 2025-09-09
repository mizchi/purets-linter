// Pure TypeScript Linter Library

use oxc_span::Span;
use std::path::{Path, PathBuf};
use crate::disable_directives::DisableDirectives;

pub mod rules;
pub mod comparer;
pub mod combined_visitor;
pub mod package_checker;
pub mod disable_directives;
mod tsconfig_validator;
mod package_json_validator;

pub use tsconfig_validator::TsConfigValidator;
pub use package_json_validator::PackageJsonValidator;
pub use package_checker::check_package_json;

pub struct Linter {
    pub path: PathBuf,
    pub source_text: String,
    pub errors: Vec<LintError>,
    pub verbose: bool,
    disable_directives: DisableDirectives,
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
        
        Self {
            path: path.to_path_buf(),
            source_text: source_text.to_string(),
            errors: Vec::new(),
            verbose,
            disable_directives,
        }
    }
    
    pub fn check_program(&mut self, program: &oxc_ast::ast::Program) {
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
        
        self.errors.push(LintError {
            rule,
            message,
            span,
        });
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