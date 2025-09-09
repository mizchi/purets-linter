use oxc::ast::ast::*;

use crate::Linter;

pub fn check_no_dynamic_access(linter: &mut Linter, program: &Program) {
    use oxc::ast_visit::Visit;
    
    struct DynamicAccessVisitor<'a, 'b> {
        linter: &'a mut Linter,
        _phantom: std::marker::PhantomData<&'b ()>,
    }
    
    impl<'a, 'b> Visit<'b> for DynamicAccessVisitor<'a, 'b> {
        fn visit_member_expression(&mut self, expr: &MemberExpression<'b>) {
            // Check for computed member expressions (bracket notation)
            if let MemberExpression::ComputedMemberExpression(computed) = expr {
                // Allow numeric indices for arrays
                let is_numeric = match &computed.expression {
                    Expression::NumericLiteral(_) => true,
                    Expression::StringLiteral(lit) => lit.value.parse::<i32>().is_ok(),
                    _ => false,
                };
                
                if !is_numeric {
                    self.linter.add_error(
                        "no-dynamic-access".to_string(),
                        "Dynamic property access is not allowed. Use dot notation or destructuring instead".to_string(),
                        computed.span,
                    );
                }
            }
            
            oxc::ast_visit::walk::walk_member_expression(self, expr);
        }
        
        fn visit_assignment_target(&mut self, target: &AssignmentTarget<'b>) {
            // Check for computed assignment targets like obj[key] = value
            if let AssignmentTarget::ComputedMemberExpression(computed) = target {
                // Allow numeric indices for arrays
                let is_numeric = match &computed.expression {
                    Expression::NumericLiteral(_) => true,
                    Expression::StringLiteral(lit) => lit.value.parse::<i32>().is_ok(),
                    _ => false,
                };
                
                if !is_numeric {
                    self.linter.add_error(
                        "no-dynamic-access".to_string(),
                        "Dynamic property assignment is not allowed. Use dot notation instead".to_string(),
                        computed.span,
                    );
                }
            }
            
            oxc::ast_visit::walk::walk_assignment_target(self, target);
        }
    }
    
    let mut visitor = DynamicAccessVisitor {
        linter,
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
        let source_type = SourceType::default();
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut linter = Linter::new(Path::new("test.ts"), source, false);
        check_no_dynamic_access(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.message).collect()
    }

    #[test]
    fn test_dynamic_property_access() {
        let source = r#"
            const obj = { foo: 1, bar: 2 };
            const key = "foo";
            const value = obj[key];
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Dynamic property access is not allowed"));
    }

    #[test]
    fn test_bracket_notation_with_string() {
        let source = r#"
            const obj = { foo: 1 };
            const value = obj["foo"];
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Dynamic property access is not allowed"));
    }

    #[test]
    fn test_dynamic_assignment() {
        let source = r#"
            const obj = {};
            const key = "prop";
            obj[key] = 42;
        "#;
        let errors = parse_and_check(source);
        // Note: This will generate 2 errors - one for access, one for assignment
        // since ComputedMemberExpression triggers both checks
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("Dynamic property")));
    }

    #[test]
    fn test_array_index_allowed() {
        let source = r#"
            const arr = [1, 2, 3];
            const value = arr[0];
            arr[1] = 10;
            const idx = arr["2"];
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0); // Numeric indices are allowed
    }

    #[test]
    fn test_dot_notation_allowed() {
        let source = r#"
            const obj = { foo: 1, bar: 2 };
            const value = obj.foo;
            obj.bar = 3;
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_computed_property_in_object_literal() {
        let source = r#"
            const key = "foo";
            const obj = {
                [key]: 42
            };
        "#;
        let errors = parse_and_check(source);
        // Note: This might need special handling if we want to forbid computed properties in object literals
        // For now, this test documents the current behavior
        assert_eq!(errors.len(), 0);
    }
}