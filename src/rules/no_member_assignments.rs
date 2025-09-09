use oxc_ast::ast::*;
use oxc_ast::visit::walk;
use oxc_ast::Visit;

use crate::Linter;

pub fn check_no_member_assignments(linter: &mut Linter, program: &Program) {
    struct MemberAssignmentChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for MemberAssignmentChecker<'a> {
        fn visit_assignment_expression(&mut self, expr: &AssignmentExpression<'a>) {
            match &expr.left {
                AssignmentTarget::StaticMemberExpression(_) | 
                AssignmentTarget::ComputedMemberExpression(_) |
                AssignmentTarget::PrivateFieldExpression(_) => {
                    self.linter.add_error(
                        "no-member-assignments".to_string(),
                        "Member assignments like 'foo.bar = value' are not allowed in pure TypeScript subset".to_string(),
                        expr.span,
                    );
                }
                _ => {}
            }
            walk::walk_assignment_expression(self, expr);
        }
    }
    
    let mut checker = MemberAssignmentChecker { linter };
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
        check_no_member_assignments(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.rule).collect()
    }

    #[test]
    fn test_member_assignment() {
        let source = r#"
            const obj = { foo: 1 };
            obj.foo = 2;
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-member-assignments".to_string()));
    }

    #[test]
    fn test_nested_member_assignment() {
        let source = r#"
            const obj = { nested: { value: 1 } };
            obj.nested.value = 2;
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-member-assignments".to_string()));
    }

    #[test]
    fn test_array_index_assignment() {
        let source = r#"
            const arr = [1, 2, 3];
            arr[0] = 4;
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-member-assignments".to_string()));
    }

    #[test]
    fn test_variable_assignment_allowed() {
        let source = r#"
            let x = 1;
            x = 2;
            
            const obj = { foo: 1 };
            const newObj = { ...obj, foo: 2 };
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_this_member_assignment() {
        let source = r#"
            class MyClass {
                value = 1;
                method() {
                    this.value = 2;
                }
            }
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-member-assignments".to_string()));
    }
}
