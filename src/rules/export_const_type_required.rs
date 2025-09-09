use oxc::ast::ast::*;
use oxc::ast_visit::walk;
use oxc::ast_visit::Visit;

use crate::Linter;

pub fn check_export_const_type_required(linter: &mut Linter, program: &Program) {
    struct ExportConstChecker<'a> {
        linter: &'a mut Linter,
    }

    impl<'a> Visit<'a> for ExportConstChecker<'a> {
        fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
            if let Some(declaration) = &decl.declaration {
                if let Declaration::VariableDeclaration(var_decl) = declaration {
                    // Check for export let (prohibited)
                    if var_decl.kind == VariableDeclarationKind::Let {
                        self.linter.add_error(
                            "no-export-let".to_string(),
                            "Export let is not allowed. Use 'export const' with explicit type "
                                .to_string(),
                            var_decl.span,
                        );
                        return;
                    }

                    // Check for export const without type annotation
                    if var_decl.kind == VariableDeclarationKind::Const {
                        for declarator in &var_decl.declarations {
                            // Check if it has a type annotation
                            if declarator.id.type_annotation.is_none() {
                                // Check if it's a function (arrow functions should have type)
                                let needs_type = if let Some(init) = &declarator.init {
                                    !matches!(
                                        init,
                                        Expression::ArrowFunctionExpression(_)
                                            | Expression::FunctionExpression(_)
                                    )
                                } else {
                                    true
                                };

                                if needs_type {
                                    // Get the name for error message
                                    let var_name = match &declarator.id.kind {
                                        BindingPatternKind::BindingIdentifier(ident) => {
                                            ident.name.to_string()
                                        }
                                        BindingPatternKind::ObjectPattern(_) => {
                                            "destructured object".to_string()
                                        }
                                        BindingPatternKind::ArrayPattern(_) => {
                                            "destructured array".to_string()
                                        }
                                        BindingPatternKind::AssignmentPattern(_) => {
                                            "assignment pattern".to_string()
                                        }
                                    };

                                    self.linter.add_error(
                                        "export-const-needs-type".to_string(),
                                        format!(
                                            "Export const '{}' must have an explicit type ",
                                            var_name
                                        ),
                                        declarator.span,
                                    );
                                }
                            }
                        }
                    }
                }
            }

            walk::walk_export_named_declaration(self, decl);
        }
    }

    let mut checker = ExportConstChecker { linter };
    checker.visit_program(program);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Linter;
    use oxc::allocator::Allocator;
    use oxc::parser::{Parser, ParserReturn};
    use oxc::span::SourceType;
    use std::path::Path;

    #[test]
    fn test_export_let_prohibited() {
        let allocator = Allocator::default();
        let source_text = r#"
export let mutableExport = "this should fail";

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);

        check_export_const_type_required(&mut linter, &program);

        let errors = &linter.errors;
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Export let is not allowed"));
    }

    #[test]
    fn test_export_const_without_type() {
        let allocator = Allocator::default();
        let source_text = r#"
export const untypedConst = "missing type";
export const untypedObject = { x: 1, y: 2 };
export const untypedArray = [1, 2, 3];

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);

        check_export_const_type_required(&mut linter, &program);

        let errors = &linter.errors;
        assert_eq!(errors.len(), 3);
        assert!(errors
            .iter()
            .all(|e| e.message.contains("must have an explicit type ")));
    }

    #[test]
    fn test_export_const_with_type_annotation() {
        let allocator = Allocator::default();
        let source_text = r#"
export const typedString: string = "typed";
export const typedObject: { x: number; y: number } = { x: 1, y: 2 };
export const typedArray: readonly number[] = [1, 2, 3];

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);

        check_export_const_type_required(&mut linter, &program);

        let errors = &linter.errors;
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_export_function_allowed() {
        let allocator = Allocator::default();
        let source_text = r#"
export function processData(data: string): string {
  return data.toUpperCase();
}

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);

        check_export_const_type_required(&mut linter, &program);

        let errors = &linter.errors;
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_arrow_function_with_types() {
        let allocator = Allocator::default();
        let source_text = r#"
export const arrowFunction: (x: number) => number = (x) => x * 2;
export const typedArrow = (x: number): number => x * 2;

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);

        check_export_const_type_required(&mut linter, &program);

        let errors = &linter.errors;
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_destructuring_without_types() {
        let allocator = Allocator::default();
        let source_text = r#"
export const { x, y } = { x: 1, y: 2 };

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);

        check_export_const_type_required(&mut linter, &program);

        let errors = &linter.errors;
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("destructured object"));
    }

    #[test]
    fn test_destructuring_with_types() {
        let allocator = Allocator::default();
        let source_text = r#"
export const { a, b }: { a: number; b: number } = { a: 1, b: 2 };

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);

        check_export_const_type_required(&mut linter, &program);

        let errors = &linter.errors;
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_multiple_violations() {
        let allocator = Allocator::default();
        let source_text = r#"
export let mutableExport = "this should fail";
export const untypedConst = "missing type";
export let first = 1, second = 2;
export const { x, y } = { x: 1, y: 2 };

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);

        check_export_const_type_required(&mut linter, &program);

        let errors = &linter.errors;
        assert!(errors.len() >= 3); // At least export let and untyped const violations
        assert!(errors
            .iter()
            .any(|e| e.message.contains("Export let is not allowed")));
        assert!(errors
            .iter()
            .any(|e| e.message.contains("must have an explicit type ")));
    }
}
