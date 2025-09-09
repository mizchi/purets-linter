use crate::Linter;
use oxc::ast::ast::*;

pub fn check_one_public_function(linter: &mut Linter, program: &Program) {
    let mut exported_functions = Vec::new();
    let mut exported_other = Vec::new();

    for item in &program.body {
        match item {
            Statement::ExportNamedDeclaration(export) => {
                if let Some(Declaration::FunctionDeclaration(func)) = &export.declaration {
                    if let Some(id) = &func.id {
                        exported_functions.push((id.name.as_str(), export.span));
                    }
                } else if let Some(Declaration::VariableDeclaration(var_decl)) = &export.declaration
                {
                    for decl in &var_decl.declarations {
                        if let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind {
                            if let Some(init) = &decl.init {
                                if matches!(
                                    init,
                                    Expression::ArrowFunctionExpression(_)
                                        | Expression::FunctionExpression(_)
                                ) {
                                    exported_functions.push((id.name.as_str(), export.span));
                                } else {
                                    exported_other.push((id.name.as_str(), export.span));
                                }
                            }
                        }
                    }
                } else if export.declaration.is_some() {
                    exported_other.push(("(declaration)", export.span));
                }

                for spec in &export.specifiers {
                    exported_other.push((spec.exported.name().as_str(), export.span));
                }
            }
            Statement::ExportDefaultDeclaration(export) => match &export.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(_) => {
                    exported_functions.push(("default", export.span));
                }
                _ if export.declaration.is_expression() => {
                    if let Some(expr) = export.declaration.as_expression() {
                        if matches!(
                            expr,
                            Expression::ArrowFunctionExpression(_)
                                | Expression::FunctionExpression(_)
                        ) {
                            exported_functions.push(("default", export.span));
                        } else {
                            exported_other.push(("default", export.span));
                        }
                    }
                }
                _ => {
                    exported_other.push(("default", export.span));
                }
            },
            _ => {}
        }
    }

    if !exported_other.is_empty() {
        for (name, span) in &exported_other {
            linter.add_error(
                "one-public-function".to_string(),
                format!(
                    "Only functions can be exported. Found non-function export: {}",
                    name
                ),
                *span,
            );
        }
    }

    if exported_functions.len() > 1 {
        for (name, span) in &exported_functions[1..] {
            linter.add_error(
                "one-public-function".to_string(),
                format!(
                    "Only one function can be exported per file. Found additional export: {}",
                    name
                ),
                *span,
            );
        }
    }
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
        let source_type = SourceType::default();
        let ret = Parser::new(&allocator, source, source_type).parse();

        let mut linter = Linter::new(Path::new("test-file.ts"), source, false);
        check_one_public_function(&mut linter, &ret.program);

        linter.errors.into_iter().map(|e| e.rule).collect()
    }

    #[test]
    fn test_single_function_export() {
        let source = r#"
            export function myFunction() {
                return 42;
            }
        "#;

        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_multiple_function_exports() {
        let source = r#"
            export function func1() {
                return 1;
            }
            
            export function func2() {
                return 2;
            }
        "#;

        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors.contains(&"one-public-function".to_string()));
    }

    #[test]
    fn test_non_function_export() {
        let source = r#"
            export const value = 42;
        "#;

        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors.contains(&"one-public-function".to_string()));
    }

    #[test]
    fn test_arrow_function_export() {
        let source = r#"
            export const myFunc = () => 42;
        "#;

        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_default_function_export() {
        let source = r#"
            export default function() {
                return 42;
            }
        "#;

        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_mixed_exports() {
        let source = r#"
            export function myFunc() {
                return 42;
            }
            
            export const value = 123;
        "#;

        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors.contains(&"one-public-function".to_string()));
    }
}
