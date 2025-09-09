use anyhow::Result;
use clap::Parser;
use colored::*;
use glob::glob;
use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_parser::{Parser as OxcParser, ParserReturn};
use oxc_span::SourceType;
use std::fs;
use std::path::{Path, PathBuf};

mod rules;
use rules::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "TypeScript file or directory to check")]
    path: String,
    
    #[arg(short, long, help = "Show detailed error messages")]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let files = collect_files(&args.path)?;
    let mut has_errors = false;
    let mut total_errors = 0;
    
    for file_path in files {
        match check_file(&file_path, args.verbose) {
            Ok(error_count) => {
                if error_count > 0 {
                    has_errors = true;
                    total_errors += error_count;
                }
            }
            Err(e) => {
                eprintln!("{}: {}", "Error".red().bold(), e);
                has_errors = true;
            }
        }
    }
    
    if has_errors {
        eprintln!("\n{} {} found", 
            "✗".red().bold(),
            format!("{} error{}", total_errors, if total_errors != 1 { "s" } else { "" }).red().bold()
        );
        std::process::exit(1);
    } else {
        println!("{} {}", "✓".green().bold(), "No errors found".green());
    }
    
    Ok(())
}

fn collect_files(path: &str) -> Result<Vec<PathBuf>> {
    let path = Path::new(path);
    let mut files = Vec::new();
    
    if path.is_file() {
        files.push(path.to_path_buf());
    } else if path.is_dir() {
        let pattern = format!("{}/**/*.ts", path.display());
        for entry in glob(&pattern)? {
            if let Ok(path) = entry {
                if !path.to_string_lossy().contains("node_modules") {
                    files.push(path);
                }
            }
        }
        
        let pattern = format!("{}/**/*.tsx", path.display());
        for entry in glob(&pattern)? {
            if let Ok(path) = entry {
                if !path.to_string_lossy().contains("node_modules") {
                    files.push(path);
                }
            }
        }
    }
    
    Ok(files)
}

fn check_file(path: &Path, verbose: bool) -> Result<usize> {
    let source_text = fs::read_to_string(path)?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or(SourceType::default());
    
    let ParserReturn {
        program,
        errors: parse_errors,
        ..
    } = OxcParser::new(&allocator, &source_text, source_type).parse();
    
    if !parse_errors.is_empty() {
        let error_count = parse_errors.len();
        for error in parse_errors {
            eprintln!("{}: Parse error: {}", 
                format!("{}:1:1", path.display()).yellow(),
                error
            );
        }
        return Ok(error_count);
    }
    
    let mut linter = Linter::new(path, &source_text, verbose);
    linter.check_program(&program);
    
    if linter.has_errors() {
        let error_count = linter.errors.len();
        linter.report_errors();
        return Ok(error_count);
    }
    
    Ok(0)
}

pub struct Linter {
    path: PathBuf,
    source_text: String,
    errors: Vec<LintError>,
    verbose: bool,
}

#[derive(Debug)]
pub struct LintError {
    pub rule: String,
    pub message: String,
    pub span: oxc_span::Span,
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
    
    pub fn check_program(&mut self, program: &Program) {
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
    }
    
    pub fn add_error(&mut self, rule: String, message: String, span: oxc_span::Span) {
        self.errors.push(LintError {
            rule,
            message,
            span,
        });
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn report_errors(&self) {
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