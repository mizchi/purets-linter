use oxc_ast::ast::*;
use oxc_ast::visit::walk;
use oxc_ast::Visit;

use crate::Linter;

pub fn check_strict_named_export(linter: &mut Linter, program: &Program) {
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
    
    struct NamedExportChecker<'a> {
        linter: &'a mut Linter,
        expected_name: String,
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
            // Check named export functions
            if let Some(Declaration::FunctionDeclaration(func)) = &decl.declaration {
                if let Some(id) = &func.id {
                    if id.name.as_str() != self.expected_name {
                        self.linter.add_error(
                            "strict-named-export".to_string(),
                            format!(
                                "Exported function '{}' must match filename '{}'",
                                id.name, self.expected_name
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
                        if id.name.as_str() != self.expected_name {
                            self.linter.add_error(
                                "strict-named-export".to_string(),
                                format!(
                                    "Exported variable '{}' must match filename '{}'",
                                    id.name, self.expected_name
                                ),
                                decl.span,
                            );
                        }
                    }
                }
            }
            
            // Check TypeScript type exports
            if let Some(Declaration::TSTypeAliasDeclaration(type_alias)) = &decl.declaration {
                if type_alias.id.name.as_str() != self.expected_name {
                    self.linter.add_error(
                        "strict-named-export".to_string(),
                        format!(
                            "Exported type '{}' must match filename '{}'",
                            type_alias.id.name, self.expected_name
                        ),
                        decl.span,
                    );
                }
            }
            
            // Check TypeScript interface exports
            if let Some(Declaration::TSInterfaceDeclaration(interface)) = &decl.declaration {
                if interface.id.name.as_str() != self.expected_name {
                    self.linter.add_error(
                        "strict-named-export".to_string(),
                        format!(
                            "Exported interface '{}' must match filename '{}'",
                            interface.id.name, self.expected_name
                        ),
                        decl.span,
                    );
                }
            }
            
            // Check export specifiers { foo } style - these are now forbidden
            if decl.declaration.is_none() && !decl.specifiers.is_empty() {
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
        expected_name,
    };
    checker.visit_program(program);
}
