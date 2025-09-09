use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub file_path: String,
    pub total_lines: usize,
    pub code_lines: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub symbol_count: usize,
    pub branch_count: usize,
    pub average_indent_depth: f64,
    pub max_indent_depth: usize,
    pub total_indent: usize,
    pub function_count: usize,
    pub class_count: usize,
    pub interface_count: usize,
    pub type_alias_count: usize,
}

impl CodeMetrics {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            total_lines: 0,
            code_lines: 0,
            blank_lines: 0,
            comment_lines: 0,
            symbol_count: 0,
            branch_count: 0,
            average_indent_depth: 0.0,
            max_indent_depth: 0,
            total_indent: 0,
            function_count: 0,
            class_count: 0,
            interface_count: 0,
            type_alias_count: 0,
        }
    }
}

impl fmt::Display for CodeMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Code Metrics for: {}", self.file_path)?;
        writeln!(f, "─────────────────────────────────────")?;
        writeln!(f, "Lines:")?;
        writeln!(f, "  Total:         {:>6}", self.total_lines)?;
        writeln!(f, "  Code:          {:>6}", self.code_lines)?;
        writeln!(f, "  Blank:         {:>6}", self.blank_lines)?;
        writeln!(f, "  Comments:      {:>6}", self.comment_lines)?;
        writeln!(f)?;
        writeln!(f, "Complexity:")?;
        writeln!(f, "  Symbols:       {:>6}", self.symbol_count)?;
        writeln!(f, "  Branches:      {:>6}", self.branch_count)?;
        writeln!(f, "  Avg Indent:    {:>6.2}", self.average_indent_depth)?;
        writeln!(f, "  Max Indent:    {:>6}", self.max_indent_depth)?;
        writeln!(f)?;
        writeln!(f, "Declarations:")?;
        writeln!(f, "  Functions:     {:>6}", self.function_count)?;
        writeln!(f, "  Classes:       {:>6}", self.class_count)?;
        writeln!(f, "  Interfaces:    {:>6}", self.interface_count)?;
        writeln!(f, "  Type Aliases:  {:>6}", self.type_alias_count)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsComparison {
    pub before: CodeMetrics,
    pub after: CodeMetrics,
    pub changes: MetricChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricChanges {
    pub total_lines_change: i32,
    pub code_lines_change: i32,
    pub symbol_count_change: i32,
    pub branch_count_change: i32,
    pub average_indent_change: f64,
    pub function_count_change: i32,
}

impl MetricChanges {
    pub fn calculate(before: &CodeMetrics, after: &CodeMetrics) -> Self {
        Self {
            total_lines_change: after.total_lines as i32 - before.total_lines as i32,
            code_lines_change: after.code_lines as i32 - before.code_lines as i32,
            symbol_count_change: after.symbol_count as i32 - before.symbol_count as i32,
            branch_count_change: after.branch_count as i32 - before.branch_count as i32,
            average_indent_change: after.average_indent_depth - before.average_indent_depth,
            function_count_change: after.function_count as i32 - before.function_count as i32,
        }
    }
}

impl fmt::Display for MetricsComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "╔═══════════════════════════════════════╗")?;
        writeln!(f, "║        Code Metrics Comparison        ║")?;
        writeln!(f, "╚═══════════════════════════════════════╝")?;
        writeln!(f)?;
        writeln!(f, "┌─────────────────┬─────────┬─────────┬─────────┐")?;
        writeln!(f, "│ Metric          │  Before │   After │  Change │")?;
        writeln!(f, "├─────────────────┼─────────┼─────────┼─────────┤")?;
        
        write_row(f, "Total Lines", 
            self.before.total_lines, 
            self.after.total_lines,
            self.changes.total_lines_change)?;
            
        write_row(f, "Code Lines", 
            self.before.code_lines, 
            self.after.code_lines,
            self.changes.code_lines_change)?;
            
        write_row(f, "Symbols", 
            self.before.symbol_count, 
            self.after.symbol_count,
            self.changes.symbol_count_change)?;
            
        write_row(f, "Branches", 
            self.before.branch_count, 
            self.after.branch_count,
            self.changes.branch_count_change)?;
            
        writeln!(f, "│ Avg Indent      │ {:>7.2} │ {:>7.2} │ {:>+7.2} │",
            self.before.average_indent_depth,
            self.after.average_indent_depth,
            self.changes.average_indent_change)?;
            
        write_row(f, "Functions", 
            self.before.function_count, 
            self.after.function_count,
            self.changes.function_count_change)?;
            
        writeln!(f, "└─────────────────┴─────────┴─────────┴─────────┘")?;
        
        writeln!(f)?;
        writeln!(f, "Summary:")?;
        
        if self.changes.code_lines_change < 0 {
            writeln!(f, "  ✓ Code reduced by {} lines", -self.changes.code_lines_change)?;
        } else if self.changes.code_lines_change > 0 {
            writeln!(f, "  → Code increased by {} lines", self.changes.code_lines_change)?;
        }
        
        if self.changes.branch_count_change < 0 {
            writeln!(f, "  ✓ Complexity reduced (branches: {})", self.changes.branch_count_change)?;
        } else if self.changes.branch_count_change > 0 {
            writeln!(f, "  → Complexity increased (branches: +{})", self.changes.branch_count_change)?;
        }
        
        if self.changes.average_indent_change < -0.1 {
            writeln!(f, "  ✓ Indentation improved by {:.2} levels", -self.changes.average_indent_change)?;
        }
        
        Ok(())
    }
}

fn write_row(f: &mut fmt::Formatter<'_>, label: &str, before: usize, after: usize, change: i32) -> fmt::Result {
    writeln!(f, "│ {:<15} │ {:>7} │ {:>7} │ {:>+7} │", label, before, after, change)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_new() {
        let metrics = CodeMetrics::new("test.ts".to_string());
        assert_eq!(metrics.file_path, "test.ts");
        assert_eq!(metrics.total_lines, 0);
        assert_eq!(metrics.code_lines, 0);
        assert_eq!(metrics.symbol_count, 0);
        assert_eq!(metrics.branch_count, 0);
    }

    #[test]
    fn test_metric_changes_calculate() {
        let mut before = CodeMetrics::new("before.ts".to_string());
        before.total_lines = 100;
        before.code_lines = 80;
        before.symbol_count = 20;
        before.branch_count = 10;
        before.average_indent_depth = 2.5;
        before.function_count = 5;

        let mut after = CodeMetrics::new("after.ts".to_string());
        after.total_lines = 75;
        after.code_lines = 60;
        after.symbol_count = 15;
        after.branch_count = 5;
        after.average_indent_depth = 2.0;
        after.function_count = 3;

        let changes = MetricChanges::calculate(&before, &after);
        
        assert_eq!(changes.total_lines_change, -25);
        assert_eq!(changes.code_lines_change, -20);
        assert_eq!(changes.symbol_count_change, -5);
        assert_eq!(changes.branch_count_change, -5);
        assert_eq!(changes.average_indent_change, -0.5);
        assert_eq!(changes.function_count_change, -2);
    }

    #[test]
    fn test_metrics_display() {
        let mut metrics = CodeMetrics::new("test.ts".to_string());
        metrics.total_lines = 100;
        metrics.code_lines = 80;
        metrics.blank_lines = 15;
        metrics.comment_lines = 5;
        
        let display = format!("{}", metrics);
        assert!(display.contains("test.ts"));
        assert!(display.contains("100"));
        assert!(display.contains("80"));
    }

    #[test]
    fn test_comparison_display() {
        let mut before = CodeMetrics::new("before.ts".to_string());
        before.total_lines = 100;
        before.code_lines = 80;
        
        let mut after = CodeMetrics::new("after.ts".to_string());
        after.total_lines = 75;
        after.code_lines = 60;
        
        let changes = MetricChanges::calculate(&before, &after);
        let comparison = MetricsComparison { before, after, changes };
        
        let display = format!("{}", comparison);
        assert!(display.contains("Code Metrics Comparison"));
        assert!(display.contains("100"));
        assert!(display.contains("75"));
        assert!(display.contains("-25"));
    }
}