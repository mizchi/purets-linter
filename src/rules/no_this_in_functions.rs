use oxc_ast::ast::*;
use oxc_ast::visit::walk;
use oxc_ast::Visit;
use oxc_syntax::scope::ScopeFlags;

use crate::Linter;

pub fn check_no_this_in_functions(linter: &mut Linter, program: &Program) {
    struct ThisChecker<'a> {
        linter: &'a mut Linter,
        in_function: bool,
        in_arrow_function: bool,
    }
    
    impl<'a> Visit<'a> for ThisChecker<'a> {
        fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
            let was_in_function = self.in_function;
            self.in_function = true;
            
            walk::walk_function(self, func, flags);
            
            self.in_function = was_in_function;
        }
        
        fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
            let was_in_arrow = self.in_arrow_function;
            self.in_arrow_function = true;
            
            walk::walk_arrow_function_expression(self, arrow);
            
            self.in_arrow_function = was_in_arrow;
        }
        
        fn visit_this_expression(&mut self, this: &ThisExpression) {
            if self.in_function || self.in_arrow_function {
                self.linter.add_error(
                    "no-this-in-functions".to_string(),
                    "Using 'this' in functions is not allowed in pure TypeScript subset".to_string(),
                    this.span,
                );
            }
        }
    }
    
    let mut checker = ThisChecker {
        linter,
        in_function: false,
        in_arrow_function: false,
    };
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
        check_no_this_in_functions(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.rule).collect()
    }

    #[test]
    fn test_this_in_function() {
        let source = r#"
            function myFunction() {
                return this.value;
            }
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-this-in-functions".to_string()));
    }

    #[test]
    fn test_this_in_arrow_function() {
        let source = r#"
            const myArrow = () => {
                return this.value;
            };
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-this-in-functions".to_string()));
    }

    #[test]
    fn test_this_in_nested_function() {
        let source = r#"
            function outer() {
                function inner() {
                    return this.value;
                }
                return inner;
            }
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-this-in-functions".to_string()));
    }

    #[test]
    fn test_this_in_object_method() {
        let source = r#"
            const obj = {
                method() {
                    return this.value;
                }
            };
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-this-in-functions".to_string()));
    }

    #[test]
    fn test_no_this_allowed() {
        let source = r#"
            function pure(x: number): number {
                return x * 2;
            }
            
            const arrow = (x: number) => x * 2;
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_this_in_class() {
        // Note: Classes themselves are not allowed, but if they were,
        // this in class methods would be a separate concern
        let source = r#"
            class MyClass {
                value = 42;
                method() {
                    return this.value;
                }
            }
        "#;
        
        let errors = parse_and_check(source);
        // Should have error for this usage
        assert!(errors.contains(&"no-this-in-functions".to_string()));
    }
}
