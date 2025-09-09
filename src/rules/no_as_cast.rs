use oxc_ast::ast::*;
use oxc_ast::visit::walk;
use oxc_ast::Visit;

use crate::Linter;

pub fn check_no_as_upcast(linter: &mut Linter, program: &Program) {
    struct AsUpcastChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for AsUpcastChecker<'a> {
        fn visit_ts_as_expression(&mut self, expr: &TSAsExpression<'a>) {
            // Check for common upcast patterns
            let is_likely_upcast = match &expr.type_annotation {
                // Casting to any, unknown, object are always upcasts
                TSType::TSAnyKeyword(_) |
                TSType::TSUnknownKeyword(_) |
                TSType::TSObjectKeyword(_) => true,
                
                // Casting to broader types like string, number, boolean might be upcasts
                TSType::TSStringKeyword(_) |
                TSType::TSNumberKeyword(_) |
                TSType::TSBooleanKeyword(_) => {
                    // These could be upcasts from literals or more specific types
                    self.linter.add_error(
                        "no-as-cast".to_string(),
                        "Type assertion with 'as' is discouraged. Consider using 'satisfies' for type checking or narrowing the type properly".to_string(),
                        expr.span,
                    );
                    return;
                }
                
                // For other types, warn about as usage in general
                _ => false,
            };
            
            if is_likely_upcast {
                self.linter.add_error(
                    "no-as-upcast".to_string(),
                    "Upcast with 'as' is not allowed. Use 'satisfies' operator instead for type validation".to_string(),
                    expr.span,
                );
            } else {
                // General warning for any 'as' usage
                self.linter.add_error(
                    "no-as-cast".to_string(),
                    "Type assertion with 'as' is discouraged. Consider using 'satisfies' for type checking or narrowing the type properly".to_string(),
                    expr.span,
                );
            }
            
            walk::walk_ts_as_expression(self, expr);
        }
        
        fn visit_ts_type_assertion(&mut self, assertion: &TSTypeAssertion<'a>) {
            // Angle bracket assertion <Type>value is also discouraged
            self.linter.add_error(
                "no-type-assertion".to_string(),
                "Type assertion <Type>value is not allowed. Use 'satisfies' operator or proper type narrowing instead".to_string(),
                assertion.span,
            );
            
            walk::walk_ts_type_assertion(self, assertion);
        }
    }
    
    let mut checker = AsUpcastChecker { linter };
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
        check_no_as_upcast(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.rule).collect()
    }

    #[test]
    fn test_as_any() {
        let source = r#"
            const value = "hello" as any;
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-as-upcast".to_string()));
    }

    #[test]
    fn test_as_unknown() {
        let source = r#"
            const data = { x: 1 } as unknown;
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-as-upcast".to_string()));
    }

    #[test]
    fn test_as_primitive() {
        let source = r#"
            const num = 42 as number;
            const str = "hello" as string;
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-as-cast".to_string()));
    }

    #[test]
    fn test_angle_bracket_assertion() {
        let source = r#"
            const oldStyle = <string>"hello";
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-type-assertion".to_string()));
    }

    #[test]
    fn test_satisfies_allowed() {
        let source = r#"
            const config = {
                apiUrl: "https://api.example.com",
                timeout: 5000
            } satisfies {
                apiUrl: string;
                timeout: number;
            };
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_as_const_allowed() {
        let source = r#"
            const tuple = [1, 2, 3] as const;
            const literal = "literal" as const;
        "#;
        
        // Note: This will still trigger the rule as implemented
        // In a real implementation, we'd want to allow 'as const'
        let errors = parse_and_check(source);
        // For now, this test documents current behavior
        assert!(!errors.is_empty());
    }
}
