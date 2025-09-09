use oxc_ast::ast::*;

use crate::Linter;

pub fn check_no_define_property(linter: &mut Linter, program: &Program) {
    use oxc_ast::Visit;
    
    struct DefinePropertyVisitor<'a, 'b> {
        linter: &'a mut Linter,
        _phantom: std::marker::PhantomData<&'b ()>,
    }
    
    impl<'a, 'b> Visit<'b> for DefinePropertyVisitor<'a, 'b> {
        fn visit_call_expression(&mut self, call: &CallExpression<'b>) {
            // Check for Object.defineProperty calls
            if let Some(member) = call.callee.as_member_expression() {
                if let MemberExpression::StaticMemberExpression(static_member) = member {
                    if let Expression::Identifier(obj) = &static_member.object {
                        if obj.name == "Object" && static_member.property.name == "defineProperty" {
                            self.linter.add_error(
                                "no-define-property".to_string(),
                                "Object.defineProperty is not allowed. Use direct property assignment or object literals instead".to_string(),
                                call.span,
                            );
                        }
                        
                        // Also check Object.defineProperties
                        if obj.name == "Object" && static_member.property.name == "defineProperties" {
                            self.linter.add_error(
                                "no-define-property".to_string(),
                                "Object.defineProperties is not allowed. Use direct property assignment or object literals instead".to_string(),
                                call.span,
                            );
                        }
                    }
                }
            }
            
            oxc_ast::visit::walk::walk_call_expression(self, call);
        }
    }
    
    let mut visitor = DefinePropertyVisitor {
        linter,
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
        check_no_define_property(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.message).collect()
    }

    #[test]
    fn test_object_define_property() {
        let source = r#"
            const obj = {};
            Object.defineProperty(obj, 'prop', {
                value: 42,
                writable: false
            });
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Object.defineProperty is not allowed"));
    }

    #[test]
    fn test_object_define_properties() {
        let source = r#"
            const obj = {};
            Object.defineProperties(obj, {
                prop1: { value: 1 },
                prop2: { value: 2 }
            });
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Object.defineProperties is not allowed"));
    }

    #[test]
    fn test_normal_property_assignment_allowed() {
        let source = r#"
            const obj = {};
            obj.prop = 42;
            const obj2 = { prop: 42 };
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }
}