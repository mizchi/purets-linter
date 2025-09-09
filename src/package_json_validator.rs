use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageJson {
    pub name: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "type")]
    pub module_type: Option<String>,
    pub main: Option<String>,
    pub scripts: Option<serde_json::Value>,
    pub dependencies: Option<serde_json::Value>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<serde_json::Value>,
}

pub struct PackageJsonValidator {
    path: String,
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl PackageJsonValidator {
    pub fn new(path: String) -> Self {
        Self {
            path,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn validate(&mut self) -> Result<()> {
        let package_json_path = if self.path.ends_with("package.json") {
            self.path.clone()
        } else {
            format!("{}/package.json", self.path)
        };
        
        let path = Path::new(&package_json_path);
        if !path.exists() {
            // package.json is optional, so no error if it doesn't exist
            return Ok(());
        }
        
        let content = fs::read_to_string(path)
            .context("Failed to read package.json")?;
        
        let package_json: PackageJson = serde_json::from_str(&content)
            .context("Failed to parse package.json")?;
        
        self.validate_module_type(&package_json);
        
        Ok(())
    }
    
    fn validate_module_type(&mut self, package_json: &PackageJson) {
        match &package_json.module_type {
            None => {
                self.errors.push("\"type\": \"module\" is missing in package.json".to_string());
            }
            Some(module_type) if module_type != "module" => {
                self.errors.push(format!(
                    "\"type\" must be \"module\", found \"{}\"", 
                    module_type
                ));
            }
            Some(_) => {
                // type: "module" is correctly set
            }
        }
    }
    
    pub fn report(&self) {
        if !self.errors.is_empty() {
            eprintln!("\n{} {} in package.json:", 
                "✗".red().bold(),
                "Errors".red().bold()
            );
            for error in &self.errors {
                eprintln!("  {} {}", "•".red(), error);
            }
        }
        
        if !self.warnings.is_empty() {
            eprintln!("\n{} {} in package.json:", 
                "⚠".yellow().bold(),
                "Warnings".yellow().bold()
            );
            for warning in &self.warnings {
                eprintln!("  {} {}", "•".yellow(), warning);
            }
        }
        
        if self.errors.is_empty() && self.warnings.is_empty() {
            println!("{} package.json validation passed", "✓".green().bold());
        }
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}