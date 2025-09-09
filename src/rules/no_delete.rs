use oxc_ast::ast::*;
use oxc_ast::visit::walk;
use oxc_ast::Visit;

use crate::Linter;

pub fn check_no_delete(linter: &mut Linter, program: &Program) {
    struct DeleteChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for DeleteChecker<'a> {
        fn visit_unary_expression(&mut self, expr: &UnaryExpression<'a>) {
            if let UnaryOperator::Delete = expr.operator {
                self.linter.add_error(
                    "no-delete".to_string(),
                    "Delete operator is not allowed in pure TypeScript subset".to_string(),
                    expr.span,
                );
            }
            walk::walk_unary_expression(self, expr);
        }
    }
    
    let mut checker = DeleteChecker { linter };
    checker.visit_program(program);
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
        check_no_delete(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.rule).collect()
    }

    #[test]
    fn test_delete_property() {
        let source = r#"
            const obj = { foo: 1, bar: 2 };
            delete obj.foo;
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-delete".to_string()));
    }

    #[test]
    fn test_delete_array_element() {
        let source = r#"
            const arr = [1, 2, 3];
            delete arr[1];
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-delete".to_string()));
    }

    #[test]
    fn test_delete_variable() {
        let source = r#"
            let x = 5;
            delete x;
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-delete".to_string()));
    }

    #[test]
    fn test_immutable_operations_allowed() {
        let source = r#"
            const obj = { foo: 1, bar: 2 };
            const { foo, ...rest } = obj;
            
            const arr = [1, 2, 3];
            const filtered = arr.filter(x => x !== 2);
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_delete_in_function() {
        let source = r#"
            function removeProperty(obj: any) {
                delete obj.prop;
            }
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-delete".to_string()));
    }
}
