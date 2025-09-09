use oxc::ast::ast::*;

use crate::Linter;

pub fn check_export_requires_jsdoc(linter: &mut Linter, program: &Program, file_path: &str) {
    use oxc::ast_visit::Visit;
    
    struct JsDocVisitor<'a, 'b> {
        linter: &'a mut Linter,
        source_text: String,
        file_path: String,
        _phantom: std::marker::PhantomData<&'b ()>,
    }
    
    impl<'a, 'b> JsDocVisitor<'a, 'b> {
        fn has_jsdoc_before(&self, span: oxc::span::Span) -> bool {
            // Check if there's a JSDoc comment immediately before this position
            let text_before = &self.source_text[..span.start as usize];
            
            // Look for JSDoc pattern (/** ... */) before the function
            // Simple check: look for */ followed by whitespace/newlines before the function
            let trimmed = text_before.trim_end();
            trimmed.ends_with("*/") && {
                // Find the start of the comment
                if let Some(_comment_start) = trimmed.rfind("/**") {
                    // Check if there's only whitespace between comment and function
                    let between = &self.source_text[trimmed.len()..span.start as usize];
                    between.trim().is_empty()
                } else {
                    false
                }
            }
        }
    }
    
    impl<'a, 'b> Visit<'b> for JsDocVisitor<'a, 'b> {
        fn visit_export_default_declaration(&mut self, export: &ExportDefaultDeclaration<'b>) {
            if let ExportDefaultDeclarationKind::FunctionDeclaration(func) = &export.declaration {
                if !self.has_jsdoc_before(export.span) {
                    let name = func.id.as_ref()
                        .map(|id| id.name.as_str())
                        .unwrap_or("anonymous");
                    self.linter.add_error(
                        "export-requires-jsdoc".to_string(),
                        format!("Exported function '{}' must have a JSDoc comment", name),
                        export.span,
                    );
                }
            }
            
            oxc::ast_visit::walk::walk_export_default_declaration(self, export);
        }
        
        fn visit_export_named_declaration(&mut self, export: &ExportNamedDeclaration<'b>) {
            if let Some(declaration) = &export.declaration {
                match declaration {
                    Declaration::FunctionDeclaration(func) => {
                        if !self.has_jsdoc_before(export.span) {
                            let name = func.id.as_ref()
                                .map(|id| id.name.as_str())
                                .unwrap_or("anonymous");
                            self.linter.add_error(
                                "export-requires-jsdoc".to_string(),
                                format!("Exported function '{}' must have a JSDoc comment", name),
                                export.span,
                            );
                        }
                    }
                    Declaration::TSTypeAliasDeclaration(type_alias) => {
                        // Check if in types/*.ts
                        if (self.file_path.contains("/types/") || self.file_path.contains("types/")) && !self.has_jsdoc_before(export.span) {
                            self.linter.add_error(
                                "export-requires-jsdoc".to_string(),
                                format!("Exported type '{}' must have a JSDoc comment", type_alias.id.name.as_str()),
                                export.span,
                            );
                        }
                    }
                    Declaration::TSInterfaceDeclaration(interface) => {
                        // Check if in types/*.ts
                        if (self.file_path.contains("/types/") || self.file_path.contains("types/")) && !self.has_jsdoc_before(export.span) {
                            self.linter.add_error(
                                "export-requires-jsdoc".to_string(),
                                format!("Exported interface '{}' must have a JSDoc comment", interface.id.name.as_str()),
                                export.span,
                            );
                        }
                    }
                    Declaration::ClassDeclaration(class) => {
                        // Check if in errors/*Error.ts
                        if (self.file_path.contains("/errors/") || self.file_path.contains("errors/")) && self.file_path.ends_with("Error.ts") && !self.has_jsdoc_before(export.span) {
                            let name = class.id.as_ref()
                                .map(|id| id.name.as_str())
                                .unwrap_or("anonymous");
                            self.linter.add_error(
                                "export-requires-jsdoc".to_string(),
                                format!("Exported error class '{}' must have a JSDoc comment", name),
                                export.span,
                            );
                        }
                    }
                    _ => {}
                }
            }
            
            oxc::ast_visit::walk::walk_export_named_declaration(self, export);
        }
    }
    
    let source_text = linter.source_text.clone();
    
    let mut visitor = JsDocVisitor {
        linter,
        source_text,
        file_path: file_path.to_string(),
        _phantom: std::marker::PhantomData,
    };
    
    visitor.visit_program(program);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Linter;
    use oxc::allocator::Allocator;
    use oxc::parser::Parser;
    use oxc::span::SourceType;
    use std::path::Path;

    fn parse_and_check(source: &str) -> Vec<String> {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new("test.ts")).unwrap();
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut linter = Linter::new(Path::new("test.ts"), source, false);
        check_export_requires_jsdoc(&mut linter, &ret.program, "test.ts");
        
        linter.errors.into_iter().map(|e| e.message).collect()
    }

    #[test]
    fn test_export_with_jsdoc() {
        let source = r#"
            /**
             * This function does something
             * @param x - The input value
             * @returns The result
             */
            export function myFunction(x: number): number {
                return x * 2;
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_export_without_jsdoc() {
        let source = r#"
            export function myFunction(x: number): number {
                return x * 2;
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("must have a JSDoc comment"));
    }

    #[test]
    fn test_default_export_with_jsdoc() {
        let source = r#"
            /**
             * Default function
             */
            export default function main() {
                console.log("hello");
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_default_export_without_jsdoc() {
        let source = r#"
            export default function main() {
                console.log("hello");
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("must have a JSDoc comment"));
    }

    #[test]
    fn test_type_with_jsdoc() {
        let source = r#"
            /**
             * Represents a user in the system
             */
            export type User = {
                id: string;
                name: string;
            };
        "#;
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new("types/User.ts")).unwrap();
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut linter = Linter::new(Path::new("types/User.ts"), source, false);
        check_export_requires_jsdoc(&mut linter, &ret.program, "types/User.ts");
        
        let errors: Vec<String> = linter.errors.into_iter().map(|e| e.message).collect();
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_type_without_jsdoc() {
        let source = r#"
            export type User = {
                id: string;
                name: string;
            };
        "#;
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new("types/User.ts")).unwrap();
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut linter = Linter::new(Path::new("types/User.ts"), source, false);
        check_export_requires_jsdoc(&mut linter, &ret.program, "types/User.ts");
        
        let errors: Vec<String> = linter.errors.into_iter().map(|e| e.message).collect();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Exported type 'User' must have a JSDoc comment"));
    }

    #[test]
    fn test_interface_with_jsdoc() {
        let source = r#"
            /**
             * Configuration interface
             */
            export interface Config {
                port: number;
                host: string;
            }
        "#;
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new("types/Config.ts")).unwrap();
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut linter = Linter::new(Path::new("types/Config.ts"), source, false);
        check_export_requires_jsdoc(&mut linter, &ret.program, "types/Config.ts");
        
        let errors: Vec<String> = linter.errors.into_iter().map(|e| e.message).collect();
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_interface_without_jsdoc() {
        let source = r#"
            export interface Config {
                port: number;
                host: string;
            }
        "#;
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new("types/Config.ts")).unwrap();
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut linter = Linter::new(Path::new("types/Config.ts"), source, false);
        check_export_requires_jsdoc(&mut linter, &ret.program, "types/Config.ts");
        
        let errors: Vec<String> = linter.errors.into_iter().map(|e| e.message).collect();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Exported interface 'Config' must have a JSDoc comment"));
    }

    #[test]
    fn test_error_class_with_jsdoc() {
        let source = r#"
            /**
             * Error thrown when file is not found
             */
            export class FileNotFoundError extends Error {
                constructor(path: string) {
                    super(`File not found: ${path}`);
                }
            }
        "#;
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new("errors/FileNotFoundError.ts")).unwrap();
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut linter = Linter::new(Path::new("errors/FileNotFoundError.ts"), source, false);
        check_export_requires_jsdoc(&mut linter, &ret.program, "errors/FileNotFoundError.ts");
        
        let errors: Vec<String> = linter.errors.into_iter().map(|e| e.message).collect();
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_error_class_without_jsdoc() {
        let source = r#"
            export class FileNotFoundError extends Error {
                constructor(path: string) {
                    super(`File not found: ${path}`);
                }
            }
        "#;
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new("errors/FileNotFoundError.ts")).unwrap();
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut linter = Linter::new(Path::new("errors/FileNotFoundError.ts"), source, false);
        check_export_requires_jsdoc(&mut linter, &ret.program, "errors/FileNotFoundError.ts");
        
        let errors: Vec<String> = linter.errors.into_iter().map(|e| e.message).collect();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Exported error class 'FileNotFoundError' must have a JSDoc comment"));
    }
}