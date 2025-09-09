use anyhow::Result;
use clap::Parser;
use colored::*;
use glob::glob;
use oxc_allocator::Allocator;
use oxc_parser::{Parser as OxcParser, ParserReturn};
use oxc_span::SourceType;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use pure_ts::{Linter, TsConfigValidator, PackageJsonValidator, comparer};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
    
    #[arg(help = "TypeScript file or directory to check")]
    path: Option<String>,
    
    #[arg(short, long, help = "Show detailed error messages")]
    verbose: bool,
    
    #[arg(long, help = "Validate tsconfig.json")]
    validate_tsconfig: bool,
    
    #[arg(short = 'j', long = "jobs", help = "Number of parallel jobs (default: CPU count)")]
    jobs: Option<usize>,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Compare code metrics between two files or directories
    Compare {
        /// Path to the original file or directory
        before: String,
        /// Path to the refactored file or directory
        after: String,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Handle compare command
    if let Some(Command::Compare { before, after }) = args.command {
        let before_path = Path::new(&before);
        let after_path = Path::new(&after);
        
        if before_path.is_file() && after_path.is_file() {
            let comparison = comparer::compare_files(before_path, after_path)?;
            println!("{}", comparison);
        } else if before_path.is_dir() && after_path.is_dir() {
            let comparisons = comparer::compare_directories(before_path, after_path)?;
            for comparison in &comparisons {
                println!("{}", comparison);
            }
            comparer::print_summary(&comparisons);
        } else {
            eprintln!("Error: Both paths must be either files or directories");
            std::process::exit(1);
        }
        return Ok(());
    }
    
    // Regular linting mode
    let path = args.path.unwrap_or_else(|| {
        eprintln!("Error: Path is required when not using compare mode");
        std::process::exit(1);
    });
    
    // Validate tsconfig.json if requested
    if args.validate_tsconfig {
        let mut tsconfig_validator = TsConfigValidator::new(path.clone());
        tsconfig_validator.validate()?;
        tsconfig_validator.report();
        
        let mut package_validator = PackageJsonValidator::new(path.clone());
        package_validator.validate()?;
        package_validator.report();
        
        if tsconfig_validator.has_errors() || package_validator.has_errors() {
            std::process::exit(1);
        }
        return Ok(());
    }
    
    let files = collect_files(&path)?;
    let file_count = files.len();
    
    if file_count == 0 {
        println!("No TypeScript files found");
        return Ok(());
    }
    
    // Configure thread pool if specified
    if let Some(jobs) = args.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .unwrap_or_else(|e| eprintln!("Warning: Failed to set thread count: {}", e));
    }
    
    let start = Instant::now();
    let total_errors = Arc::new(AtomicUsize::new(0));
    let verbose = args.verbose;
    
    // Process files in parallel using rayon
    let _results: Vec<_> = files
        .par_iter()
        .map(|file_path| {
            match check_file(file_path, verbose) {
                Ok(error_count) => {
                    if error_count > 0 {
                        total_errors.fetch_add(error_count, Ordering::Relaxed);
                    }
                    Ok(error_count)
                }
                Err(e) => {
                    eprintln!("{}: {}", "Error".red().bold(), e);
                    total_errors.fetch_add(1, Ordering::Relaxed);
                    Err(e)
                }
            }
        })
        .collect();
    
    let duration = start.elapsed();
    let total_errors = total_errors.load(Ordering::Relaxed);
    let has_errors = total_errors > 0;
    
    if has_errors {
        eprintln!("\n{} {} found in {:.2}s", 
            "✗".red().bold(),
            format!("{} error{}", total_errors, if total_errors != 1 { "s" } else { "" }).red().bold(),
            duration.as_secs_f64()
        );
        std::process::exit(1);
    } else {
        println!("{} {} in {} file{} ({:.2}s, {:.0} files/sec)", 
            "✓".green().bold(), 
            "No errors found".green(),
            file_count,
            if file_count != 1 { "s" } else { "" },
            duration.as_secs_f64(),
            file_count as f64 / duration.as_secs_f64()
        );
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