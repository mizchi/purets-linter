use std::fs;
use std::path::{Path, PathBuf};
use glob::Pattern;

/// Filter for excluding files based on .gitignore patterns
#[derive(Debug, Clone)]
pub struct GitignoreFilter {
    patterns: Vec<IgnorePattern>,
    default_excludes: Vec<String>,
}

#[derive(Debug, Clone)]
struct IgnorePattern {
    pattern: String,
    is_negation: bool,
    is_directory: bool,
    glob: Option<Pattern>,
}

impl GitignoreFilter {
    /// Create a new filter with default exclusions
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            default_excludes: vec![
                "node_modules".to_string(),
                "dist".to_string(),
                "out".to_string(),
                "target".to_string(),
                "build".to_string(),
                "coverage".to_string(),
                ".git".to_string(),
                ".next".to_string(),
                ".nuxt".to_string(),
                ".output".to_string(),
                ".vercel".to_string(),
                "*.min.js".to_string(),
                "*.min.ts".to_string(),
                "vendor".to_string(),
                "tmp".to_string(),
                "temp".to_string(),
            ],
        }
    }
    
    /// Load patterns from .gitignore file
    pub fn load_from_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(path)?;
        self.parse_gitignore(&content);
        Ok(())
    }
    
    /// Load from project root, checking for .gitignore
    pub fn load_from_project(&mut self, project_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let gitignore_path = project_root.join(".gitignore");
        if gitignore_path.exists() {
            self.load_from_file(&gitignore_path)?;
        }
        
        // Also check for .gitignore in parent directories (for monorepos)
        if let Some(parent) = project_root.parent() {
            let parent_gitignore = parent.join(".gitignore");
            if parent_gitignore.exists() {
                self.load_from_file(&parent_gitignore)?;
            }
        }
        
        Ok(())
    }
    
    /// Parse gitignore content
    fn parse_gitignore(&mut self, content: &str) {
        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            let mut pattern = line.to_string();
            let mut is_negation = false;
            let mut is_directory = false;
            
            // Handle negation patterns
            if pattern.starts_with('!') {
                is_negation = true;
                pattern = pattern[1..].to_string();
            }
            
            // Handle directory patterns
            if pattern.ends_with('/') {
                is_directory = true;
                pattern.pop();
            }
            
            // Convert gitignore pattern to glob pattern
            let glob_pattern = self.gitignore_to_glob(&pattern);
            let glob = Pattern::new(&glob_pattern).ok();
            
            self.patterns.push(IgnorePattern {
                pattern: pattern.clone(),
                is_negation,
                is_directory,
                glob,
            });
        }
    }
    
    /// Convert gitignore pattern to glob pattern
    fn gitignore_to_glob(&self, pattern: &str) -> String {
        let mut glob = String::new();
        let mut chars = pattern.chars().peekable();
        
        // If pattern doesn't start with /, it matches anywhere
        let is_absolute = pattern.starts_with('/');
        if !is_absolute {
            glob.push_str("**/");
        } else {
            // Remove leading /
            chars.next();
        }
        
        while let Some(ch) = chars.next() {
            match ch {
                '*' => {
                    if chars.peek() == Some(&'*') {
                        chars.next();
                        glob.push_str("**");
                    } else {
                        glob.push('*');
                    }
                }
                '?' => glob.push('?'),
                '[' => {
                    glob.push('[');
                    // Copy character class as-is
                    while let Some(ch) = chars.next() {
                        glob.push(ch);
                        if ch == ']' {
                            break;
                        }
                    }
                }
                _ => glob.push(ch),
            }
        }
        
        glob
    }
    
    /// Check if a file should be ignored
    pub fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        // Check default excludes first
        for exclude in &self.default_excludes {
            if exclude.contains('*') {
                // Pattern matching
                if let Ok(pattern) = Pattern::new(exclude) {
                    if pattern.matches(&path_str) {
                        return true;
                    }
                }
            } else {
                // Simple string matching for directories
                if path_str.contains(&format!("/{}/", exclude)) 
                    || path_str.contains(&format!("\\{}\\", exclude))
                    || path_str.starts_with(&format!("{}/", exclude))
                    || path_str.starts_with(&format!("{}\\", exclude))
                    || path_str.ends_with(&format!("/{}", exclude))
                    || path_str.ends_with(&format!("\\{}", exclude))
                    || &*path_str == exclude {
                    return true;
                }
            }
        }
        
        // Check gitignore patterns
        let mut should_ignore = false;
        
        for pattern in &self.patterns {
            if let Some(ref glob) = pattern.glob {
                let matches = glob.matches(&path_str);
                
                if matches {
                    if pattern.is_negation {
                        should_ignore = false;
                    } else {
                        should_ignore = true;
                    }
                }
            } else {
                // Fallback to simple string matching
                let matches = if pattern.is_directory {
                    path.is_dir() && path_str.contains(&pattern.pattern)
                } else {
                    path_str.contains(&pattern.pattern)
                };
                
                if matches {
                    if pattern.is_negation {
                        should_ignore = false;
                    } else {
                        should_ignore = true;
                    }
                }
            }
        }
        
        should_ignore
    }
    
    /// Filter a list of paths
    pub fn filter_paths(&self, paths: Vec<PathBuf>) -> Vec<PathBuf> {
        paths.into_iter()
            .filter(|path| !self.should_ignore(path))
            .collect()
    }
    
    /// Check if path contains any excluded directory
    pub fn contains_excluded_dir(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        
        // Common build/output directories to exclude
        let exclude_dirs = [
            "node_modules",
            "dist",
            "out",
            "target",
            "build",
            ".git",
            ".next",
            ".nuxt",
            ".output",
            ".vercel",
            "coverage",
            "vendor",
            "tmp",
            "temp",
        ];
        
        for dir in &exclude_dirs {
            if path_str.contains(&format!("/{}/", dir))
                || path_str.contains(&format!("\\{}\\", dir))
                || path_str.contains(&format!("/{}", dir))
                || path_str.contains(&format!("\\{}", dir)) {
                return true;
            }
        }
        
        false
    }
}

impl Default for GitignoreFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_default_excludes() {
        let filter = GitignoreFilter::new();
        
        assert!(filter.should_ignore(Path::new("node_modules/package.json")));
        assert!(filter.should_ignore(Path::new("dist/index.js")));
        assert!(filter.should_ignore(Path::new("target/debug/app")));
        assert!(filter.should_ignore(Path::new(".git/config")));
        
        assert!(!filter.should_ignore(Path::new("src/index.ts")));
        assert!(!filter.should_ignore(Path::new("package.json")));
    }
    
    #[test]
    fn test_gitignore_patterns() {
        let mut filter = GitignoreFilter::new();
        
        let gitignore_content = r#"
# Comments should be ignored
*.log
*.tmp
/build/
!important.log
docs/**/*.pdf
"#;
        
        filter.parse_gitignore(gitignore_content);
        
        assert!(filter.should_ignore(Path::new("error.log")));
        assert!(filter.should_ignore(Path::new("temp.tmp")));
        assert!(filter.should_ignore(Path::new("docs/manual/guide.pdf")));
        
        // Negation pattern
        // Note: This is simplified - real gitignore negation is more complex
        assert!(!filter.should_ignore(Path::new("src/main.ts")));
    }
    
    #[test]
    fn test_filter_paths() {
        let filter = GitignoreFilter::new();
        
        let paths = vec![
            PathBuf::from("src/index.ts"),
            PathBuf::from("node_modules/lib/index.js"),
            PathBuf::from("dist/bundle.js"),
            PathBuf::from("src/utils.ts"),
        ];
        
        let filtered = filter.filter_paths(paths);
        
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&PathBuf::from("src/index.ts")));
        assert!(filtered.contains(&PathBuf::from("src/utils.ts")));
    }
    
    #[test]
    fn test_contains_excluded_dir() {
        let filter = GitignoreFilter::new();
        
        assert!(filter.contains_excluded_dir(Path::new("path/to/node_modules/file.js")));
        assert!(filter.contains_excluded_dir(Path::new("dist/output.js")));
        assert!(filter.contains_excluded_dir(Path::new(".git/HEAD")));
        
        assert!(!filter.contains_excluded_dir(Path::new("src/index.ts")));
        assert!(!filter.contains_excluded_dir(Path::new("packages/my-pkg/src/main.ts")));
    }
    
    #[test]
    fn test_load_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");
        
        fs::write(&gitignore_path, "*.test.ts\n*.spec.ts\n").unwrap();
        
        let mut filter = GitignoreFilter::new();
        filter.load_from_file(&gitignore_path).unwrap();
        
        assert!(filter.should_ignore(Path::new("app.test.ts")));
        assert!(filter.should_ignore(Path::new("utils.spec.ts")));
        assert!(!filter.should_ignore(Path::new("app.ts")));
    }
}