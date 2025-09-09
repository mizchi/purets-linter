use oxc_ast::ast::*;
use oxc_ast::Comment;

use crate::Linter;

pub fn check_export_requires_jsdoc(linter: &mut Linter, program: &Program) {
    use oxc_ast::Visit;
    
    struct JsDocVisitor<'a, 'b> {
        linter: &'a mut Linter,
        source_text: String,
        _phantom: std::marker::PhantomData<&'b ()>,
    }
    
    impl<'a, 'b> JsDocVisitor<'a, 'b> {
        fn has_jsdoc_before(&self, span: oxc_span::Span) -> bool {
            // Check if there's a JSDoc comment immediately before this position
            let text_before = &self.source_text[..span.start as usize];
            
            // Look for JSDoc pattern (/** ... */) before the function
            // Simple check: look for */ followed by whitespace/newlines before the function
            let trimmed = text_before.trim_end();
            trimmed.ends_with("*/") && {
                // Find the start of the comment
                if let Some(comment_start) = trimmed.rfind("/**") {
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
            match &export.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
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
                _ => {}
            }
            
            oxc_ast::visit::walk::walk_export_default_declaration(self, export);
        }
        
        fn visit_export_named_declaration(&mut self, export: &ExportNamedDeclaration<'b>) {
            if let Some(Declaration::FunctionDeclaration(func)) = &export.declaration {
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
            
            oxc_ast::visit::walk::walk_export_named_declaration(self, export);
        }
    }
    
    let source_text = linter.source_text.clone();
    
    let mut visitor = JsDocVisitor {
        linter,
        source_text,
        _phantom: std::marker::PhantomData,
    };
    
    visitor.visit_program(program);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Linter;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;
    use std::path::Path;

    fn parse_and_check(source: &str) -> Vec<String> {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut linter = Linter::new(Path::new("test.ts"), source, false);
        check_export_requires_jsdoc(&mut linter, &ret.program);
        
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
}