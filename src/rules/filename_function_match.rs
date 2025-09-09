use oxc_ast::ast::*;
use std::path::Path;

use crate::Linter;

pub fn check_filename_function_match(linter: &mut Linter, program: &Program) {
    use oxc_ast::Visit;
    
    // Skip type definition files in types/ directory and error classes first
    let path_str = linter.path.to_str().unwrap_or("").replace('\\', "/");
    if linter.verbose {
        eprintln!("DEBUG filename-function-match: path_str = {}", path_str);
    }
    if path_str.contains("/types/") || path_str.contains("/errors/") {
        if linter.verbose {
            eprintln!("DEBUG filename-function-match: Skipping types/errors file");
        }
        return;
    }
    
    // Get filename without extension
    let filename = linter.path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    
    // Remove leading underscore for private files
    let expected_name = if filename.starts_with('_') {
        &filename[1..]
    } else {
        filename
    };
    
    // Skip index files and test files
    if filename == "index" || filename.ends_with(".test") || filename.ends_with(".spec") || filename.ends_with("_test") {
        return;
    }
    
    struct FilenameMatchVisitor<'a, 'b> {
        linter: &'a mut Linter,
        filename: String,
        expected_name: String,
        found_matching_export: bool,
        _phantom: std::marker::PhantomData<&'b ()>,
    }
    
    impl<'a, 'b> Visit<'b> for FilenameMatchVisitor<'a, 'b> {
        fn visit_export_default_declaration(&mut self, export: &ExportDefaultDeclaration<'b>) {
            // Check default export function
            match &export.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                    if let Some(id) = &func.id {
                        if id.name.as_str() == self.expected_name {
                            self.found_matching_export = true;
                        } else {
                            self.linter.add_error(
                                "filename-function-match".to_string(),
                                format!(
                                    "Exported function name '{}' must match filename '{}'",
                                    id.name, self.expected_name
                                ),
                                export.span,
                            );
                        }
                    }
                }
                ExportDefaultDeclarationKind::Identifier(ident) => {
                    if ident.name.as_str() != self.expected_name {
                        self.linter.add_error(
                            "filename-function-match".to_string(),
                            format!(
                                "Exported identifier '{}' must match filename '{}'",
                                ident.name, self.expected_name
                            ),
                            export.span,
                        );
                    } else {
                        self.found_matching_export = true;
                    }
                }
                _ => {}
            }
            
            oxc_ast::visit::walk::walk_export_default_declaration(self, export);
        }
        
        fn visit_export_named_declaration(&mut self, export: &ExportNamedDeclaration<'b>) {
            // Check named export functions
            if let Some(Declaration::FunctionDeclaration(func)) = &export.declaration {
                if let Some(id) = &func.id {
                    if id.name.as_str() == self.expected_name {
                        self.found_matching_export = true;
                    }
                }
            }
            
            // Check export specifiers - simplified for now
            // TODO: Check individual export specifiers
            
            oxc_ast::visit::walk::walk_export_named_declaration(self, export);
        }
    }
    
    let filename_str = filename.to_string();
    let expected_name_str = expected_name.to_string();
    
    let mut visitor = FilenameMatchVisitor {
        linter,
        filename: filename_str.clone(),
        expected_name: expected_name_str.clone(),
        found_matching_export: false,
        _phantom: std::marker::PhantomData,
    };
    
    visitor.visit_program(program);
    
    // If we have exports but none match the filename, that's an error
    if !visitor.found_matching_export && !filename_str.is_empty() {
        // Check if there are any exports at all
        let has_exports = program.body.iter().any(|stmt| {
            matches!(stmt, Statement::ExportDefaultDeclaration(_) | 
                          Statement::ExportNamedDeclaration(_))
        });
        
        if has_exports {
            visitor.linter.add_error(
                "filename-function-match".to_string(),
                format!("File '{}' must export a function with the same name '{}'", filename_str, expected_name_str),
                oxc_span::Span::new(0, 0),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Linter;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;
    use std::path::Path;

    fn parse_and_check(source: &str, filename: &str) -> Vec<String> {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut linter = Linter::new(Path::new(filename), source, false);
        check_filename_function_match(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.message).collect()
    }

    #[test]
    fn test_matching_default_export() {
        let source = r#"
            export default function myFunction() {
                return 42;
            }
        "#;
        let errors = parse_and_check(source, "myFunction.ts");
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_mismatched_default_export() {
        let source = r#"
            export default function wrongName() {
                return 42;
            }
        "#;
        let errors = parse_and_check(source, "myFunction.ts");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("must match filename"));
    }

    #[test]
    fn test_matching_named_export() {
        let source = r#"
            export function myFunction() {
                return 42;
            }
        "#;
        let errors = parse_and_check(source, "myFunction.ts");
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_index_file_exempt() {
        let source = r#"
            export function anyName() {
                return 42;
            }
        "#;
        let errors = parse_and_check(source, "index.ts");
        assert_eq!(errors.len(), 0);
    }
}