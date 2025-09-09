use crate::Linter;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

pub fn check_no_top_level_side_effects(linter: &mut Linter, program: &Program) {
    // Allow main() calls in main.ts files
    let path_str = linter.path.to_str().unwrap_or("");
    let is_main_file = path_str.ends_with("/main.ts") || 
                       path_str.ends_with("\\main.ts") || 
                       path_str == "main.ts";
    
    if linter.verbose && is_main_file {
        eprintln!("DEBUG: Detected main.ts file: {}", path_str);
    }
    
    for item in &program.body {
        match item {
            Statement::ExpressionStatement(expr_stmt) => match &expr_stmt.expression {
                Expression::CallExpression(call) => {
                    // Allow main() calls in main.ts
                    if is_main_file && is_main_function_call(call) {
                        continue;
                    }
                    
                    // Allow Deno.test() calls in test files when using deno-test runner
                    if is_deno_test_call(call) && linter.test_runner == Some(crate::TestRunner::DenoTest) {
                        continue;
                    }
                    
                    if !is_iife(call) {
                        linter.add_error(
                            "no-top-level-side-effects".to_string(),
                            "Top-level function calls are not allowed (side effects)".to_string(),
                            expr_stmt.span,
                        );
                    }
                }
                Expression::AssignmentExpression(_) => {
                    linter.add_error(
                        "no-top-level-side-effects".to_string(),
                        "Top-level assignments are not allowed (side effects)".to_string(),
                        expr_stmt.span,
                    );
                }
                Expression::UpdateExpression(_) => {
                    linter.add_error(
                        "no-top-level-side-effects".to_string(),
                        "Top-level update expressions are not allowed (side effects)".to_string(),
                        expr_stmt.span,
                    );
                }
                Expression::NewExpression(_) => {
                    linter.add_error(
                        "no-top-level-side-effects".to_string(),
                        "Top-level new expressions are not allowed (side effects)".to_string(),
                        expr_stmt.span,
                    );
                }
                _ => {}
            },
            Statement::ForStatement(_)
            | Statement::ForInStatement(_)
            | Statement::ForOfStatement(_)
            | Statement::WhileStatement(_)
            | Statement::DoWhileStatement(_) => {
                linter.add_error(
                    "no-top-level-side-effects".to_string(),
                    "Top-level loops are not allowed (side effects)".to_string(),
                    item.span(),
                );
            }
            Statement::IfStatement(if_stmt) => {
                if !is_type_guard_only(if_stmt) {
                    linter.add_error(
                        "no-top-level-side-effects".to_string(),
                        "Top-level if statements are not allowed (side effects)".to_string(),
                        if_stmt.span,
                    );
                }
            }
            _ => {}
        }
    }
}

fn is_iife(call: &CallExpression) -> bool {
    match &call.callee {
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => true,
        Expression::ParenthesizedExpression(paren) => {
            matches!(
                &paren.expression,
                Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
            )
        }
        _ => false,
    }
}

fn is_main_function_call(call: &CallExpression) -> bool {
    match &call.callee {
        Expression::Identifier(id) => id.name == "main",
        _ => false,
    }
}

fn is_deno_test_call(call: &CallExpression) -> bool {
    match &call.callee {
        Expression::StaticMemberExpression(member) => {
            if let Expression::Identifier(obj) = &member.object {
                obj.name == "Deno" && member.property.name == "test"
            } else {
                false
            }
        }
        _ => false,
    }
}

fn is_type_guard_only(_if_stmt: &IfStatement) -> bool {
    false
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
        
        let mut linter = Linter::new(Path::new("test-file.ts"), source, false);
        check_no_top_level_side_effects(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.rule).collect()
    }

    #[test]
    fn test_top_level_function_call() {
        let source = r#"
            console.log("hello");
            myFunction();
        "#;
        
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().all(|e| e == "no-top-level-side-effects"));
    }

    #[test]
    fn test_iife_allowed() {
        let source = r#"
            (() => {
                console.log("hello");
            })();
            
            (function() {
                console.log("world");
            })();
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_top_level_assignment() {
        let source = r#"
            let x = 5;
            x = 10;
        "#;
        
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors.contains(&"no-top-level-side-effects".to_string()));
    }

    #[test]
    fn test_top_level_loops() {
        let source = r#"
            for (let i = 0; i < 10; i++) {
                console.log(i);
            }
            
            while (true) {
                break;
            }
        "#;
        
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().all(|e| e == "no-top-level-side-effects"));
    }

    #[test]
    fn test_top_level_new_expression() {
        let source = r#"
            new Date();
        "#;
        
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors.contains(&"no-top-level-side-effects".to_string()));
    }

    #[test]
    fn test_no_side_effects() {
        let source = r#"
            const x = 5;
            function myFunction() {
                return 42;
            }
            export { myFunction };
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }
}
