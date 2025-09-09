use crate::Linter;
use oxc::ast::ast::*;
use oxc::ast_visit::Visit;

pub fn check_no_do_while(linter: &mut Linter, program: &Program) {
    struct DoWhileChecker<'a> {
        linter: &'a mut Linter,
    }

    impl<'a> Visit<'a> for DoWhileChecker<'a> {
        fn visit_do_while_statement(&mut self, stmt: &DoWhileStatement<'a>) {
            self.linter.add_error(
                "no-do-while".to_string(),
                "do-while statements are not allowed. Use while instead".to_string(),
                stmt.span,
            );
        }
    }

    let mut checker = DoWhileChecker { linter };
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
        let source_type = SourceType::from_path("test.ts").unwrap();
        let ret = Parser::new(&allocator, source, source_type).parse();

        let mut linter = Linter::new(Path::new("test.ts"), source, false);
        check_no_do_while(&mut linter, &ret.program);

        linter.errors.into_iter().map(|e| e.rule).collect()
    }

    #[test]
    fn test_do_while_statement() {
        let source = r#"
            let i = 0;
            do {
                i++;
            } while (i < 10);
        "#;

        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors.contains(&"no-do-while".to_string()));
    }

    #[test]
    fn test_while_allowed() {
        let source = r#"
            let i = 0;
            while (i < 10) {
                i++;
            }
        "#;

        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_nested_do_while() {
        let source = r#"
            function test() {
                do {
                    console.log("test");
                } while (false);
            }
        "#;

        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors.contains(&"no-do-while".to_string()));
    }
}
