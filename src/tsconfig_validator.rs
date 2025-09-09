use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsConfig {
    #[serde(rename = "compilerOptions")]
    pub compiler_options: Option<CompilerOptions>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub extends: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilerOptions {
    pub strict: Option<bool>,
    pub no_implicit_any: Option<bool>,
    pub no_implicit_this: Option<bool>,
    pub always_strict: Option<bool>,
    pub strict_null_checks: Option<bool>,
    pub strict_function_types: Option<bool>,
    pub strict_bind_call_apply: Option<bool>,
    pub strict_property_initialization: Option<bool>,
    pub no_implicit_returns: Option<bool>,
    pub no_fallthrough_cases_in_switch: Option<bool>,
    pub no_unused_locals: Option<bool>,
    pub no_unused_parameters: Option<bool>,
    pub exact_optional_property_types: Option<bool>,
    pub no_unchecked_indexed_access: Option<bool>,
    pub no_property_access_from_index_signature: Option<bool>,
    pub allow_unreachable_code: Option<bool>,
    pub allow_unused_labels: Option<bool>,
    pub allow_import_ts_extension: Option<bool>,
    pub verbatim_module_syntax: Option<bool>,
    pub module: Option<String>,
    pub target: Option<String>,
}

pub struct TsConfigValidator {
    path: String,
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl TsConfigValidator {
    pub fn new(path: String) -> Self {
        Self {
            path,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn validate(&mut self) -> Result<()> {
        let tsconfig_path = if self.path.ends_with("tsconfig.json") {
            self.path.clone()
        } else {
            format!("{}/tsconfig.json", self.path)
        };

        let path = Path::new(&tsconfig_path);
        if !path.exists() {
            self.errors
                .push(format!("tsconfig.json not found at {}", tsconfig_path));
            return Ok(());
        }

        let content = fs::read_to_string(path).context("Failed to read tsconfig.json")?;

        // Parse as raw JSON first to check for unknown properties
        let _raw_json: Value =
            serde_json::from_str(&content).context("Invalid JSON in tsconfig.json")?;

        // Parse into struct
        let tsconfig: TsConfig =
            serde_json::from_str(&content).context("Failed to parse tsconfig.json structure")?;

        self.validate_compiler_options(&tsconfig.compiler_options);
        self.validate_required_settings(&tsconfig);

        Ok(())
    }

    fn validate_compiler_options(&mut self, options: &Option<CompilerOptions>) {
        match options {
            None => {
                self.errors
                    .push("compilerOptions is missing in tsconfig.json".to_string());
            }
            Some(opts) => {
                // Check strict mode
                if opts.strict != Some(true) {
                    self.errors.push("strict must be set to true".to_string());
                }

                // If strict is not true, check individual strict options
                if opts.strict != Some(true) {
                    let strict_options = vec![
                        (opts.no_implicit_any, "noImplicitAny"),
                        (opts.no_implicit_this, "noImplicitThis"),
                        (opts.always_strict, "alwaysStrict"),
                        (opts.strict_null_checks, "strictNullChecks"),
                        (opts.strict_function_types, "strictFunctionTypes"),
                        (opts.strict_bind_call_apply, "strictBindCallApply"),
                        (
                            opts.strict_property_initialization,
                            "strictPropertyInitialization",
                        ),
                    ];

                    for (option, name) in strict_options {
                        if option != Some(true) {
                            self.warnings.push(format!(
                                "{} should be true when strict is not enabled",
                                name
                            ));
                        }
                    }
                }

                // Recommend additional strict options
                if opts.no_implicit_returns != Some(true) {
                    self.warnings
                        .push("Consider enabling noImplicitReturns for safer code".to_string());
                }

                if opts.no_fallthrough_cases_in_switch != Some(true) {
                    self.warnings
                        .push("Consider enabling noFallthroughCasesInSwitch".to_string());
                }

                if opts.no_unused_locals != Some(true) {
                    self.warnings
                        .push("Consider enabling noUnusedLocals".to_string());
                }

                if opts.no_unused_parameters != Some(true) {
                    self.errors
                        .push("noUnusedParameters must be set to true".to_string());
                }

                if opts.exact_optional_property_types != Some(true) {
                    self.warnings.push(
                        "Consider enabling exactOptionalPropertyTypes for stricter typing"
                            .to_string(),
                    );
                }

                if opts.no_unchecked_indexed_access != Some(true) {
                    self.warnings.push(
                        "Consider enabling noUncheckedIndexedAccess for safer array/object access"
                            .to_string(),
                    );
                }

                // Check for problematic settings
                if opts.allow_unreachable_code == Some(true) {
                    self.errors
                        .push("allowUnreachableCode should not be true".to_string());
                }

                if opts.allow_unused_labels == Some(true) {
                    self.errors
                        .push("allowUnusedLabels should not be true".to_string());
                }

                // Check required settings for .ts extension imports
                if opts.allow_import_ts_extension != Some(true) {
                    self.errors
                        .push("allowImportTsExtension must be set to true".to_string());
                }

                if opts.verbatim_module_syntax != Some(true) {
                    self.errors
                        .push("verbatimModuleSyntax must be set to true".to_string());
                }

                // Check module and target
                if let Some(module) = &opts.module {
                    if module != "ESNext" && module != "ES2022" && module != "ES2020" {
                        self.warnings.push(format!(
                            "Consider using ESNext or ES2022 for module, currently: {}",
                            module
                        ));
                    }
                }

                if let Some(target) = &opts.target {
                    if target != "ESNext" && target != "ES2022" && target != "ES2020" {
                        self.warnings.push(format!(
                            "Consider using ESNext or ES2022 for target, currently: {}",
                            target
                        ));
                    }
                }
            }
        }
    }

    fn validate_required_settings(&mut self, tsconfig: &TsConfig) {
        // Check if extends is used (which might override settings)
        if let Some(extends) = &tsconfig.extends {
            self.warnings.push(format!(
                "Using extends '{}' - make sure it doesn't override strict settings",
                extends
            ));
        }
    }

    pub fn report(&self) {
        if !self.errors.is_empty() {
            eprintln!(
                "\n{} {} in tsconfig.json:",
                "✗".red().bold(),
                "Errors".red().bold()
            );
            for error in &self.errors {
                eprintln!("  {} {}", "•".red(), error);
            }
        }

        if !self.warnings.is_empty() {
            eprintln!(
                "\n{} {} in tsconfig.json:",
                "⚠".yellow().bold(),
                "Warnings".yellow().bold()
            );
            for warning in &self.warnings {
                eprintln!("  {} {}", "•".yellow(), warning);
            }
        }

        if self.errors.is_empty() && self.warnings.is_empty() {
            println!("{} tsconfig.json validation passed", "✓".green().bold());
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    #[allow(dead_code)]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}
