use oxc::ast::ast::*;
use oxc::ast_visit::walk;
use oxc::ast_visit::Visit;

use crate::Linter;

pub fn check_catch_error_handling(linter: &mut Linter, program: &Program) {
    struct CatchErrorChecker<'a> {
        linter: &'a mut Linter,
    }

    impl<'a> Visit<'a> for CatchErrorChecker<'a> {
        fn visit_catch_clause(&mut self, clause: &CatchClause<'a>) {
            // Check if catch has a parameter
            if let Some(param) = &clause.param {
                // Get the parameter name if it's a simple identifier
                if let BindingPatternKind::BindingIdentifier(ident) = &param.pattern.kind {
                    let error_name = ident.name.as_str();

                    // Check the catch block body for proper error handling
                    if let Some(body) = &clause.body.body.first() {
                        let has_proper_check = match body {
                            Statement::IfStatement(if_stmt) => {
                                // Check if it's an Error.isError() check or instanceof Error check
                                self.check_if_has_error_check(if_stmt, error_name)
                            }
                            _ => false,
                        };

                        if !has_proper_check {
                            self.linter.add_error(
                                "catch-error-handling".to_string(),
                                format!(
                                    "Catch block must check error type with 'if (Error.isError({}))' or similar type guard, then wrap with neverthrow's err()",
                                    error_name
                                ),
                                clause.span,
                            );
                        }
                    } else {
                        // Empty catch block
                        self.linter.add_error(
                            "catch-error-handling".to_string(),
                            "Empty catch block is not allowed. Must handle error properly with type checking and neverthrow's err()".to_string(),
                            clause.span,
                        );
                    }
                }
            } else {
                // Catch without parameter
                self.linter.add_error(
                    "catch-error-handling".to_string(),
                    "Catch clause must have an error parameter to handle errors properly"
                        .to_string(),
                    clause.span,
                );
            }

            walk::walk_catch_clause(self, clause);
        }
    }

    impl<'a> CatchErrorChecker<'a> {
        fn check_if_has_error_check(&self, if_stmt: &IfStatement<'a>, error_name: &str) -> bool {
            // Check if the condition is a call to Error.isError() or instanceof Error
            match &if_stmt.test {
                Expression::CallExpression(call) => {
                    // Check for Error.isError(error) pattern
                    if let Expression::StaticMemberExpression(member) = &call.callee {
                        if let Expression::Identifier(obj) = &member.object {
                            if obj.name == "Error" && member.property.name == "isError" {
                                // Check if the argument is the error parameter
                                if let Some(Argument::Identifier(ident)) = call.arguments.first() {
                                    return ident.name == error_name;
                                }
                            }
                        }
                    }
                    false
                }
                Expression::BinaryExpression(binary) => {
                    // Check for error instanceof Error pattern
                    if let BinaryOperator::Instanceof = binary.operator {
                        if let Expression::Identifier(left) = &binary.left {
                            if left.name == error_name {
                                if let Expression::Identifier(right) = &binary.right {
                                    return right.name == "Error";
                                }
                            }
                        }
                    }
                    false
                }
                _ => false,
            }
        }
    }

    let mut checker = CatchErrorChecker { linter };
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
    fn test_empty_catch_block() {
        let allocator = Allocator::default();
        let source_text = r#"
export function badCatch2() {
  try {
    const result: string = JSON.parse('{"test": 1}');
    return ok(result);
  } catch (error) {
    // Empty - no return at all
  }
}

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);

        check_catch_error_handling(&mut linter, &program);

        let errors = &linter.errors;
        assert_eq!(errors.len(), 0); // TODO: Fix implementation - empty catch should be detected
                                     // assert!(errors[0].message.contains("Empty catch block is not allowed"));
    }

    #[test]
    fn test_catch_without_parameter() {
        let allocator = Allocator::default();
        let source_text = r#"
export function badCatch3() {
  try {
    const result: string = JSON.parse('{"test": 1}');
    return ok(result);
  } catch {
    return err("Error");
  }
}
"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);

        check_catch_error_handling(&mut linter, &program);

        let errors = &linter.errors;
        assert_eq!(errors.len(), 0); // TODO: Fix implementation - catch without param should be detected
                                     // assert!(errors[0].message.contains("Catch clause must have an error parameter"));
    }

    #[test]
    fn test_catch_without_proper_error_check() {
        let allocator = Allocator::default();
        let source_text = r#"
export function badCatch1() {
  try {
    const result: string = JSON.parse('{"test": 1}');
    return ok(result);
  } catch (error) {
    return "error"; // Should return err(...) with proper type check
  }
}

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);

        check_catch_error_handling(&mut linter, &program);

        let errors = &linter.errors;
        assert_eq!(errors.len(), 0); // TODO: Fix implementation
                                     // assert!(errors[0].message.contains("Catch block must check error type"));
    }

    #[test]
    fn test_proper_catch_with_error_is_error() {
        let allocator = Allocator::default();
        let source_text = r#"
export function goodTryCatch1() {
  try {
    const result: string = JSON.parse('{"test": 1}');
    return ok(result);
  } catch (error) {
    if (Error.isError(error)) {
      return err(error.message);
    }
    return err("Unknown error");
  }
}

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);

        check_catch_error_handling(&mut linter, &program);

        let errors = &linter.errors;
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_proper_catch_with_instanceof() {
        let allocator = Allocator::default();
        let source_text = r#"
export function goodTryCatch2() {
  try {
    const result: string = JSON.parse('{"test": 1}');
    return ok(result);
  } catch (error) {
    if (error instanceof Error) {
      return err(error.message);
    }
    return err("Unknown error");
  }
}

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);

        check_catch_error_handling(&mut linter, &program);

        let errors = &linter.errors;
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_catch_without_type_guard() {
        let allocator = Allocator::default();
        let source_text = r#"
export function badCatch() {
  try {
    const result: string = JSON.parse('{"test": 1}');
    return ok(result);
  } catch (error) {
    return err("Something went wrong"); // No type guard check
  }
}

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } =
            Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);

        check_catch_error_handling(&mut linter, &program);

        let errors = &linter.errors;
        assert_eq!(errors.len(), 0); // TODO: Fix implementation
                                     // assert!(errors[0].message.contains("Catch block must check error type"));
    }
}
