use anyhow::Result;
use clap::Parser;
use colored::*;
use glob::glob;
use oxc_allocator::Allocator;
use oxc_parser::{Parser as OxcParser, ParserReturn};
use oxc_span::SourceType;
use std::fs;
use std::path::{Path, PathBuf};

use pure_ts::{Linter, TsConfigValidator, PackageJsonValidator};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "TypeScript file or directory to check")]
    path: String,
    
    #[arg(short, long, help = "Show detailed error messages")]
    verbose: bool,
    
    #[arg(long, help = "Validate tsconfig.json")]
    validate_tsconfig: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Validate tsconfig.json if requested
    if args.validate_tsconfig {
        let mut tsconfig_validator = TsConfigValidator::new(args.path.clone());
        tsconfig_validator.validate()?;
        tsconfig_validator.report();
        
        let mut package_validator = PackageJsonValidator::new(args.path.clone());
        package_validator.validate()?;
        package_validator.report();
        
        if tsconfig_validator.has_errors() || package_validator.has_errors() {
            std::process::exit(1);
        }
        return Ok(());
    }
    
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