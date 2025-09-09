use anyhow::Result;
use clap::Parser;
use colored::*;
use glob::glob;
use oxc::allocator::Allocator;
use oxc::parser::{Parser as OxcParser, ParserReturn};
use oxc::span::SourceType;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use purets::{
    check_package_json, comparer,
    gitignore_filter::GitignoreFilter,
    test_runner_detector::{TestRunner as DetectedTestRunner, TestRunnerDetector},
    workspace_detector::WorkspaceConfig,
    Linter, PackageJsonValidator, TestRunner, TsConfigValidator,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(help = "TypeScript file or directory to check (defaults to current directory)")]
    path: Option<String>,

    #[arg(short, long, help = "Show detailed error messages")]
    verbose: bool,

    #[arg(long, help = "Validate tsconfig.json")]
    validate_tsconfig: bool,

    #[arg(
        short = 'j',
        long = "jobs",
        help = "Number of parallel jobs (default: CPU count)"
    )]
    jobs: Option<usize>,

    #[arg(
        long = "test",
        help = "Test runner to use (vitest, node-test, deno-test)"
    )]
    test: Option<String>,

    #[arg(
        long = "entry",
        help = "Mark files as entry points (allows re-exports)",
        value_delimiter = ','
    )]
    entry: Vec<String>,

    #[arg(
        long = "main",
        help = "Mark files as main entry points",
        value_delimiter = ','
    )]
    main: Vec<String>,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Initialize a new pure-ts project
    New {
        /// Path where the project will be created
        path: String,
    },
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

    // Handle subcommands
    if let Some(command) = args.command {
        match command {
            Command::New { path } => {
                let project_path = Path::new(&path);
                purets::init::init_project(project_path)?;
                return Ok(());
            }
            Command::Compare { before, after } => {
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
        }
    }

    // Regular linting mode - default to current directory
    let path = args.path.unwrap_or_else(|| ".".to_string());

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

    // Detect workspace configuration
    let project_path = if Path::new(&path).is_file() {
        Path::new(&path).parent().unwrap_or(Path::new("."))
    } else {
        Path::new(&path)
    };

    let workspace_config = WorkspaceConfig::detect(project_path);

    if workspace_config.is_monorepo() {
        println!(
            "{}",
            format!(
                "Detected {} workspace with {} package patterns",
                match workspace_config.workspace_type {
                    purets::workspace_detector::WorkspaceType::PnpmWorkspaces => "pnpm",
                    purets::workspace_detector::WorkspaceType::NpmWorkspaces => "npm",
                    purets::workspace_detector::WorkspaceType::YarnWorkspaces => "yarn",
                    _ => "unknown",
                },
                workspace_config.packages.len()
            )
            .cyan()
        );
    }

    // Check package.json for forbidden dependencies
    let package_errors = check_package_json(project_path);
    if !package_errors.is_empty() {
        eprintln!("{}", "Package.json dependency errors:".red().bold());
        for error in &package_errors {
            eprintln!("  {}", error.red());
        }
    }

    let files = if Path::new(&path).is_file() {
        // Single file specified
        vec![Path::new(&path).to_path_buf()]
    } else {
        // Use workspace-aware file collection
        collect_files_with_workspace(&workspace_config)?
    };
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

    // Parse test runner if specified, or auto-detect
    let test_runner = if let Some(test_str) = &args.test {
        match TestRunner::from_str(test_str) {
            Some(runner) => {
                println!("Using test runner: {}", runner);
                Some(runner)
            }
            None => {
                eprintln!(
                    "Error: Unknown test runner '{}'. Valid options: vitest, node-test, deno-test",
                    test_str
                );
                std::process::exit(1);
            }
        }
    } else {
        // Auto-detect test runner
        let detector =
            TestRunnerDetector::new(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let detected = detector.detect();
        match detected {
            DetectedTestRunner::Vitest => {
                println!("Auto-detected test runner: {}", "vitest".cyan());
                Some(TestRunner::Vitest)
            }
            DetectedTestRunner::NodeTest => {
                println!("Auto-detected test runner: {}", "node-test".cyan());
                Some(TestRunner::NodeTest)
            }
            DetectedTestRunner::DenoTest => {
                println!("Auto-detected test runner: {}", "deno-test".cyan());
                Some(TestRunner::DenoTest)
            }
            DetectedTestRunner::None => None,
        }
    };

    let start = Instant::now();
    let total_errors = Arc::new(AtomicUsize::new(0));
    let verbose = args.verbose;

    // Convert entry and main paths to absolute paths for comparison
    let entry_paths: Vec<PathBuf> = args
        .entry
        .iter()
        .map(|p| {
            Path::new(p)
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(p))
        })
        .collect();
    let main_paths: Vec<PathBuf> = args
        .main
        .iter()
        .map(|p| {
            Path::new(p)
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(p))
        })
        .collect();

    // Process files in parallel using rayon
    let _results: Vec<_> = files
        .par_iter()
        .map(|file_path| {
            let runner = test_runner.clone();
            // Compare canonical paths or check if the file path ends with the entry/main path
            let is_entry = entry_paths.iter().any(|ep| {
                let matches = file_path == ep
                    || ep
                        .file_name()
                        .map_or(false, |name| file_path.ends_with(name));
                if verbose && matches {
                    eprintln!("DEBUG: Marking {} as entry point", file_path.display());
                }
                matches
            });
            let is_main = main_paths.iter().any(|mp| {
                let matches = file_path == mp
                    || mp
                        .file_name()
                        .map_or(false, |name| file_path.ends_with(name));
                if verbose && matches {
                    eprintln!("DEBUG: Marking {} as main entry", file_path.display());
                }
                matches
            });
            match check_file_with_options(file_path, verbose, runner, is_entry, is_main) {
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
    let file_errors = total_errors.load(Ordering::Relaxed);
    let total_errors = file_errors + package_errors.len();
    let has_errors = total_errors > 0;

    if has_errors {
        eprintln!(
            "\n{} {} found in {:.2}s",
            "✗".red().bold(),
            format!(
                "{} error{}",
                total_errors,
                if total_errors != 1 { "s" } else { "" }
            )
            .red()
            .bold(),
            duration.as_secs_f64()
        );
        std::process::exit(1);
    } else {
        println!(
            "{} {} in {} file{} ({:.2}s, {:.0} files/sec)",
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

fn collect_files_with_workspace(workspace: &WorkspaceConfig) -> Result<Vec<PathBuf>> {
    let mut all_files = Vec::new();

    // Initialize gitignore filter
    let mut filter = GitignoreFilter::new();
    filter.load_from_project(&workspace.root).ok();

    // Get all target directories from workspace
    let target_dirs = workspace.get_target_dirs();

    if workspace.is_monorepo() {
        println!("Scanning {} package directories...", target_dirs.len());
    }

    for dir in target_dirs {
        let files = collect_files(dir.to_str().unwrap_or("."))?;
        all_files.extend(files);
    }

    // Remove duplicates and sort
    all_files.sort();
    all_files.dedup();

    // Apply gitignore filtering
    let filtered_files = filter.filter_paths(all_files);

    Ok(filtered_files)
}

fn collect_files(path: &str) -> Result<Vec<PathBuf>> {
    let path = Path::new(path);
    let mut files = Vec::new();
    let filter = GitignoreFilter::new();

    if path.is_file() {
        files.push(path.to_path_buf());
    } else if path.is_dir() {
        let pattern = format!("{}/**/*.ts", path.display());
        for entry in glob(&pattern)? {
            if let Ok(path) = entry {
                // Use gitignore filter instead of simple node_modules check
                if !filter.contains_excluded_dir(&path) {
                    files.push(path);
                }
            }
        }

        let pattern = format!("{}/**/*.tsx", path.display());
        for entry in glob(&pattern)? {
            if let Ok(path) = entry {
                if !filter.contains_excluded_dir(&path) {
                    files.push(path);
                }
            }
        }
    }

    Ok(files)
}

// Removed unused functions - functionality consolidated into check_file_with_options

fn check_file_with_options(
    path: &Path,
    verbose: bool,
    test_runner: Option<TestRunner>,
    is_entry: bool,
    is_main: bool,
) -> Result<usize> {
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
            eprintln!(
                "{}: Parse error: {}",
                format!("{}:1:1", path.display()).yellow(),
                error
            );
        }
        return Ok(error_count);
    }

    let mut linter = Linter::new(path, &source_text, verbose)
        .with_test_runner(test_runner)
        .with_entry_point(is_entry)
        .with_main_entry(is_main);
    linter.check_program(&program);

    // Check for untriggered expect-error directives
    linter.check_untriggered_expect_errors();

    if linter.has_errors() {
        let error_count = linter.errors.len();
        linter.report_errors();
        return Ok(error_count);
    }

    Ok(0)
}
