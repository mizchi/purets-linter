// Pure TypeScript Linter Library

use oxc_span::Span;
use std::path::{Path, PathBuf};

pub mod rules;
pub mod comparer;
mod tsconfig_validator;
mod package_json_validator;

pub use tsconfig_validator::TsConfigValidator;
pub use package_json_validator::PackageJsonValidator;

pub struct Linter {
    pub path: PathBuf,
    pub source_text: String,
    pub errors: Vec<LintError>,
    pub verbose: bool,
}

#[derive(Debug)]
pub struct LintError {
    pub rule: String,
    pub message: String,
    pub span: Span,
}

impl Linter {
    pub fn new(path: &Path, source_text: &str, verbose: bool) -> Self {
        Self {
            path: path.to_path_buf(),
            source_text: source_text.to_string(),
            errors: Vec::new(),
            verbose,
        }
    }
    
    pub fn check_program(&mut self, program: &oxc_ast::ast::Program) {
        use crate::rules::*;
        
        check_no_classes(self, program);
        check_no_enums(self, program);
        check_no_reexports(self, program);
        check_no_namespace_imports(self, program);
        check_no_member_assignments(self, program);
        check_one_public_function(self, program);
        check_no_top_level_side_effects(self, program);
        check_no_unused_variables(self, program);
        check_import_extensions(self, program);
        check_no_getters_setters(self, program);
        check_must_use_return_value(self, program);
        check_no_delete(self, program);
        check_no_this_in_functions(self, program);
        check_no_throw(self, program);
        check_no_foreach(self, program);
        check_no_filename_dirname(self, program);
        check_interface_extends_only(self, program);
        check_no_eval_function(self, program);
        check_no_object_assign(self, program);
        check_no_constant_condition(self, program);
        check_switch_case_block(self, program);
        check_no_as_upcast(self, program);
        check_let_requires_type(self, program);
        check_catch_error_handling(self, program);
        check_jsdoc_param_match(self, program);
        check_no_named_exports(self, program);
        check_export_const_type_required(self, program);
        check_no_unused_map(self, program);
        check_no_do_while(self, program);
        check_no_mutable_record(self, program);
        check_empty_array_requires_type(self, program);
    }
    
    pub fn add_error(&mut self, rule: String, message: String, span: Span) {
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