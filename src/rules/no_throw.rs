use oxc::ast::ast::*;
use oxc::ast_visit::walk;
use oxc::ast_visit::Visit;

use crate::Linter;

pub fn check_no_throw(linter: &mut Linter, program: &Program) {
    struct ThrowChecker<'a> {
        linter: &'a mut Linter,
    }

    impl<'a> Visit<'a> for ThrowChecker<'a> {
        fn visit_throw_statement(&mut self, stmt: &ThrowStatement<'a>) {
            self.linter.add_error(
                "no-throw".to_string(),
                "Throwing exceptions is not allowed. Use Result type from neverthrow instead"
                    .to_string(),
                stmt.span,
            );
        }

        fn visit_try_statement(&mut self, stmt: &TryStatement<'a>) {
            // First report that try-catch is not allowed
            self.linter.add_error(
                "no-try-catch".to_string(),
                "Try-catch blocks are not allowed. Use Result type from neverthrow instead"
                    .to_string(),
                stmt.span,
            );

            // But if try-catch is used, ensure it returns ok() in try and err() in catch
            self.check_try_block_returns(&stmt.block);

            if let Some(handler) = &stmt.handler {
                self.check_catch_block_returns(&handler.body);
            }

            walk::walk_try_statement(self, stmt);
        }
    }

    impl<'a> ThrowChecker<'a> {
        fn check_try_block_returns(&mut self, block: &BlockStatement<'a>) {
            let has_ok_return = self.block_returns_ok(block);
            if !has_ok_return {
                self.linter.add_error(
                    "try-must-return-ok".to_string(),
                    "Try block must return ok(...) from neverthrow".to_string(),
                    block.span,
                );
            }
        }

        fn check_catch_block_returns(&mut self, block: &BlockStatement<'a>) {
            let has_err_return = self.block_returns_err(block);
            if !has_err_return {
                self.linter.add_error(
                    "catch-must-return-err".to_string(),
                    "Catch block must return err(...) from neverthrow".to_string(),
                    block.span,
                );
            }
        }

        fn block_returns_ok(&self, block: &BlockStatement<'a>) -> bool {
            for stmt in &block.body {
                if let Statement::ReturnStatement(ret) = stmt {
                    if let Some(arg) = &ret.argument {
                        if self.is_ok_call(arg) {
                            return true;
                        }
                    }
                }
            }
            false
        }

        fn block_returns_err(&self, block: &BlockStatement<'a>) -> bool {
            for stmt in &block.body {
                if let Statement::ReturnStatement(ret) = stmt {
                    if let Some(arg) = &ret.argument {
                        if self.is_err_call(arg) {
                            return true;
                        }
                    }
                }
            }
            false
        }

        fn is_ok_call(&self, expr: &Expression<'a>) -> bool {
            if let Expression::CallExpression(call) = expr {
                if let Expression::Identifier(ident) = &call.callee {
                    return ident.name == "ok";
                }
            }
            false
        }

        fn is_err_call(&self, expr: &Expression<'a>) -> bool {
            if let Expression::CallExpression(call) = expr {
                if let Expression::Identifier(ident) = &call.callee {
                    return ident.name == "err";
                }
            }
            false
        }
    }

    let mut checker = ThrowChecker { linter };
    checker.visit_program(program);
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
        check_no_throw(&mut linter, &ret.program);

        linter.errors.into_iter().map(|e| e.rule).collect()
    }

    #[test]
    fn test_throw_statement() {
        let source = r#"
            function doSomething() {
                throw new Error("Something went wrong");
            }
        "#;

        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-throw".to_string()));
    }

    #[test]
    fn test_try_catch() {
        let source = r#"
            function handleError() {
                try {
                    doSomething();
                } catch (error) {
                    console.error(error);
                }
            }
        "#;

        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-try-catch".to_string()));
    }

    #[test]
    fn test_try_without_ok() {
        let source = r#"
            function badTry() {
                try {
                    const result = JSON.parse('{}');
                    return result;
                } catch (error) {
                    return err("error");
                }
            }
        "#;

        let errors = parse_and_check(source);
        assert!(errors.contains(&"try-must-return-ok".to_string()));
    }

    #[test]
    fn test_catch_without_err() {
        let source = r#"
            function badCatch() {
                try {
                    return ok("success");
                } catch (error) {
                    return "error";
                }
            }
        "#;

        let errors = parse_and_check(source);
        assert!(errors.contains(&"catch-must-return-err".to_string()));
    }

    #[test]
    fn test_proper_try_catch() {
        let source = r#"
            function goodTryCatch() {
                try {
                    const result = doSomething();
                    return ok(result);
                } catch (error) {
                    return err("Something failed");
                }
            }
        "#;

        let errors = parse_and_check(source);
        // Should still have no-try-catch error, but not the return errors
        assert!(errors.contains(&"no-try-catch".to_string()));
        assert!(!errors.contains(&"try-must-return-ok".to_string()));
        assert!(!errors.contains(&"catch-must-return-err".to_string()));
    }

    #[test]
    fn test_result_type_usage() {
        let source = r#"
            import { ok, err } from 'neverthrow';
            
            function divide(a: number, b: number) {
                if (b === 0) {
                    return err("Division by zero");
                }
                return ok(a / b);
            }
        "#;

        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }
}
