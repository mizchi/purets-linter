pub mod metrics;
pub mod analyzer;

use anyhow::Result;
use std::path::Path;

pub use metrics::{CodeMetrics, MetricsComparison, MetricChanges};
pub use analyzer::CodeAnalyzer;

pub fn compare_files(before_path: &Path, after_path: &Path) -> Result<MetricsComparison> {
    let before_metrics = CodeAnalyzer::analyze_file(before_path)?;
    let after_metrics = CodeAnalyzer::analyze_file(after_path)?;
    
    let changes = MetricChanges::calculate(&before_metrics, &after_metrics);
    
    Ok(MetricsComparison {
        before: before_metrics,
        after: after_metrics,
        changes,
    })
}

pub fn compare_directories(before_dir: &Path, after_dir: &Path) -> Result<Vec<MetricsComparison>> {
    use std::collections::HashMap;
    
    let mut comparisons = Vec::new();
    
    // Get all TypeScript files from both directories
    let before_files = collect_ts_files(before_dir)?;
    let after_files = collect_ts_files(after_dir)?;
    
    // Create a map for easier lookup
    let mut after_map: HashMap<String, _> = HashMap::new();
    for file in after_files {
        let relative = file.strip_prefix(after_dir)?;
        after_map.insert(relative.to_string_lossy().to_string(), file);
    }
    
    // Compare matching files
    for before_file in before_files {
        let relative = before_file.strip_prefix(before_dir)?;
        let relative_str = relative.to_string_lossy().to_string();
        
        if let Some(after_file) = after_map.get(&relative_str) {
            match compare_files(&before_file, after_file) {
                Ok(comparison) => comparisons.push(comparison),
                Err(e) => eprintln!("Error comparing {}: {}", relative_str, e),
            }
        }
    }
    
    Ok(comparisons)
}

fn collect_ts_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    use glob::glob;
    
    let pattern = format!("{}/**/*.{{ts,tsx}}", dir.display());
    let mut files = Vec::new();
    
    for path in (glob(&pattern)?).flatten() {
        if !path.to_string_lossy().contains("node_modules") {
            files.push(path);
        }
    }
    
    Ok(files)
}

pub fn print_summary(comparisons: &[MetricsComparison]) {
    use colored::*;
    
    let mut total_before = CodeMetrics::new("Total".to_string());
    let mut total_after = CodeMetrics::new("Total".to_string());
    
    for comp in comparisons {
        total_before.total_lines += comp.before.total_lines;
        total_before.code_lines += comp.before.code_lines;
        total_before.symbol_count += comp.before.symbol_count;
        total_before.branch_count += comp.before.branch_count;
        total_before.function_count += comp.before.function_count;
        
        total_after.total_lines += comp.after.total_lines;
        total_after.code_lines += comp.after.code_lines;
        total_after.symbol_count += comp.after.symbol_count;
        total_after.branch_count += comp.after.branch_count;
        total_after.function_count += comp.after.function_count;
    }
    
    let total_changes = MetricChanges::calculate(&total_before, &total_after);
    
    println!("\n{}", "═".repeat(50).blue());
    println!("{}", "OVERALL SUMMARY".blue().bold());
    println!("{}", "═".repeat(50).blue());
    
    println!("\nFiles analyzed: {}", comparisons.len());
    
    println!("\n{:<20} {:>10} {:>10} {:>10}", 
        "Metric".bold(), 
        "Before".bold(), 
        "After".bold(), 
        "Change".bold()
    );
    println!("{}", "─".repeat(50));
    
    print_metric_row("Total Lines", 
        total_before.total_lines, 
        total_after.total_lines,
        total_changes.total_lines_change);
        
    print_metric_row("Code Lines", 
        total_before.code_lines, 
        total_after.code_lines,
        total_changes.code_lines_change);
        
    print_metric_row("Symbols", 
        total_before.symbol_count, 
        total_after.symbol_count,
        total_changes.symbol_count_change);
        
    print_metric_row("Branches", 
        total_before.branch_count, 
        total_after.branch_count,
        total_changes.branch_count_change);
        
    print_metric_row("Functions", 
        total_before.function_count, 
        total_after.function_count,
        total_changes.function_count_change);
    
    println!("\n{}", "Key Improvements:".green().bold());
    
    if total_changes.code_lines_change < 0 {
        println!("  {} Code reduced by {} lines ({}%)",
            "✓".green(),
            -total_changes.code_lines_change,
            ((-total_changes.code_lines_change as f64 / total_before.code_lines as f64) * 100.0) as i32
        );
    }
    
    if total_changes.branch_count_change < 0 {
        println!("  {} Complexity reduced by {} branches",
            "✓".green(),
            -total_changes.branch_count_change
        );
    }
    
    if total_changes.symbol_count_change < 0 {
        println!("  {} Symbol count reduced by {}",
            "✓".green(),
            -total_changes.symbol_count_change
        );
    }
}

fn print_metric_row(label: &str, before: usize, after: usize, change: i32) {
    use colored::*;
    
    let change_str = if change > 0 {
        format!("+{}", change).red()
    } else if change < 0 {
        format!("{}", change).green()
    } else {
        format!("{}", change).white()
    };
    
    println!("{:<20} {:>10} {:>10} {:>10}", 
        label, 
        before, 
        after,
        change_str
    );
}