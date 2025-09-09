use oxc::ast::ast::*;
use oxc::ast_visit::walk;
use oxc::ast_visit::Visit;

use crate::Linter;

/// Unified rule for filename-export matching
/// Combines:
/// - strict-named-export: Prohibits export default, requires named exports
/// - filename-function-match: General filename matching
/// - path-based-restrictions: Directory-specific rules
pub fn check_strict_named_export(linter: &mut Linter, program: &Program) {
    let path_str = linter.path.to_str().unwrap_or("").replace('\\', "/");
    
    // Get filename without extension and without leading underscore
    let filename = linter.path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    
    let expected_name = if filename.starts_with('_') {
        filename[1..].to_string()
    } else {
        filename.clone()
    };
    
    // Skip index files and test files
    if filename == "index" || filename.ends_with(".test") || filename.ends_with(".spec") || filename.ends_with("_test") {
        return;
    }
    
    // Skip main.ts and entry points
    if filename == "main" || linter.is_entry_point || linter.is_main_entry {
        return;
    }
    
    // Determine file type based on path
    let file_type = if path_str.contains("/types/") {
        FileType::TypeDefinition
    } else if path_str.contains("/errors/") {
        FileType::ErrorClass
    } else if path_str.contains("/pure/") {
        FileType::PureFunction
    } else if path_str.contains("/io/") {
        FileType::IoFunction
    } else {
        FileType::Regular
    };
    
    struct NamedExportChecker<'a> {
        linter: &'a mut Linter,
        expected_name: String,
        _filename: String,
        file_type: FileType,
        found_matching_export: bool,
        export_count: usize,
    }
    
    impl<'a> Visit<'a> for NamedExportChecker<'a> {
        fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
            // Export default is not allowed
            self.linter.add_error(
                "strict-named-export".to_string(),
                "Export default is not allowed. Use named export matching the filename".to_string(),
                decl.span,
            );
            
            walk::walk_export_default_declaration(self, decl);
        }
        
        fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
            self.export_count += 1;
            
            // Check named export functions
            if let Some(Declaration::FunctionDeclaration(func)) = &decl.declaration {
                if let Some(id) = &func.id {
                    let name = id.name.as_str();
                    
                    // For IO functions, also check if they are async
                    if self.file_type == FileType::IoFunction
                        && !func.r#async && !name.ends_with("Sync") {
                            self.linter.add_error(
                                "strict-named-export".to_string(),
                                format!("IO function '{}' must be async or end with 'Sync'", name),
                                decl.span,
                            );
                        }
                    
                    // For pure functions, check they are not async
                    if self.file_type == FileType::PureFunction && func.r#async {
                        self.linter.add_error(
                            "strict-named-export".to_string(),
                            format!("Pure function '{}' cannot be async", name),
                            decl.span,
                        );
                    }
                    
                    if name == self.expected_name {
                        self.found_matching_export = true;
                    } else if self.file_type != FileType::TypeDefinition && self.file_type != FileType::ErrorClass {
                        self.linter.add_error(
                            "strict-named-export".to_string(),
                            format!(
                                "Exported function '{}' must match filename '{}'",
                                name, self.expected_name
                            ),
                            decl.span,
                        );
                    }
                }
            }
            
            // Check named export const/let/var
            if let Some(Declaration::VariableDeclaration(var_decl)) = &decl.declaration {
                for declarator in &var_decl.declarations {
                    if let BindingPatternKind::BindingIdentifier(id) = &declarator.id.kind {
                        let name = id.name.as_str();
                        if name == self.expected_name {
                            self.found_matching_export = true;
                        } else if self.file_type != FileType::TypeDefinition && self.file_type != FileType::ErrorClass {
                            self.linter.add_error(
                                "strict-named-export".to_string(),
                                format!(
                                    "Exported variable '{}' must match filename '{}'",
                                    name, self.expected_name
                                ),
                                decl.span,
                            );
                        }
                    }
                }
            }
            
            // Check TypeScript type exports
            if let Some(Declaration::TSTypeAliasDeclaration(type_alias)) = &decl.declaration {
                let name = type_alias.id.name.as_str();
                
                if self.file_type == FileType::TypeDefinition {
                    if name == self.expected_name {
                        self.found_matching_export = true;
                    } else {
                        self.linter.add_error(
                            "strict-named-export".to_string(),
                            format!(
                                "Type export must be named '{}' to match the filename",
                                self.expected_name
                            ),
                            decl.span,
                        );
                    }
                } else if name != self.expected_name {
                    self.linter.add_error(
                        "strict-named-export".to_string(),
                        format!(
                            "Exported type '{}' must match filename '{}'",
                            name, self.expected_name
                        ),
                        decl.span,
                    );
                }
            }
            
            // Check TypeScript interface exports
            if let Some(Declaration::TSInterfaceDeclaration(interface)) = &decl.declaration {
                let name = interface.id.name.as_str();
                
                if self.file_type == FileType::TypeDefinition {
                    if name == self.expected_name {
                        self.found_matching_export = true;
                    } else {
                        self.linter.add_error(
                            "strict-named-export".to_string(),
                            format!(
                                "Interface export must be named '{}' to match the filename",
                                self.expected_name
                            ),
                            decl.span,
                        );
                    }
                } else if name != self.expected_name {
                    self.linter.add_error(
                        "strict-named-export".to_string(),
                        format!(
                            "Exported interface '{}' must match filename '{}'",
                            name, self.expected_name
                        ),
                        decl.span,
                    );
                }
            }
            
            // Check class exports (only for errors/)
            if let Some(Declaration::ClassDeclaration(class)) = &decl.declaration {
                if let Some(id) = &class.id {
                    let name = id.name.as_str();
                    
                    if self.file_type == FileType::ErrorClass {
                        if name == self.expected_name {
                            self.found_matching_export = true;
                        } else {
                            self.linter.add_error(
                                "strict-named-export".to_string(),
                                format!(
                                    "Error class must be named '{}' to match filename",
                                    self.expected_name
                                ),
                                decl.span,
                            );
                        }
                    }
                }
            }
            
            // Check export specifiers { foo } style - these are now forbidden (except for re-exports in index files)
            if decl.declaration.is_none() && !decl.specifiers.is_empty() && decl.source.is_none() {
                self.linter.add_error(
                    "strict-named-export".to_string(),
                    "Export specifier syntax 'export { }' is not allowed. Use direct export declarations like 'export function' or 'export type'".to_string(),
                    decl.span,
                );
            }
            
            walk::walk_export_named_declaration(self, decl);
        }
    }
    
    let mut checker = NamedExportChecker { 
        linter,
        expected_name: expected_name.clone(),
        _filename: filename.clone(),
        file_type,
        found_matching_export: false,
        export_count: 0,
    };
    checker.visit_program(program);
    
    // Check if we found the matching export (skip for certain file types that have their own rules)
    if checker.export_count > 0 && !checker.found_matching_export {
        let message = match checker.file_type {
            FileType::TypeDefinition => {
                format!("types/**/*.ts must export a type named '{}' matching the filename", expected_name)
            },
            FileType::ErrorClass => {
                format!("errors/**/*.ts must export a class named '{}' matching the filename", expected_name)
            },
            FileType::PureFunction => {
                format!("pure/**/*.ts must export a function named '{}' matching the filename", expected_name)
            },
            FileType::IoFunction => {
                format!("io/**/*.ts must export a function named '{}' matching the filename", expected_name)
            },
            FileType::Regular => {
                format!("File '{}' must export a function with the same name '{}'", filename, expected_name)
            },
        };
        
        checker.linter.add_error(
            "strict-named-export".to_string(),
            message,
            oxc::span::Span::new(0, 0),
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FileType {
    Regular,
    TypeDefinition,
    ErrorClass,
    PureFunction,
    IoFunction,
}