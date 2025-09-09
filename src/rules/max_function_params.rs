use oxc::ast::ast::*;

use crate::Linter;

const MAX_PARAMS: usize = 2;

pub fn check_max_function_params(linter: &mut Linter, program: &Program) {
    use oxc::ast_visit::Visit;
    use oxc::syntax::scope::ScopeFlags;
    
    struct MaxParamsVisitor<'a, 'b> {
        linter: &'a mut Linter,
        _phantom: std::marker::PhantomData<&'b ()>,
    }
    
    impl<'a, 'b> Visit<'b> for MaxParamsVisitor<'a, 'b> {
        fn visit_function(&mut self, func: &Function<'b>, _flags: ScopeFlags) {
            let param_count = func.params.items.len();
            if param_count > MAX_PARAMS {
                let func_name = func.id.as_ref()
                    .map(|id| id.name.as_str())
                    .unwrap_or("<anonymous>");
                
                self.linter.add_error(
                    "max-function-params".to_string(),
                    format!(
                        "Function '{}' has {} parameters (max: {}). Use an options object as the second parameter instead",
                        func_name, param_count, MAX_PARAMS
                    ),
                    func.span,
                );
            }
            
            oxc::ast_visit::walk::walk_function(self, func, _flags);
        }
        
        fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'b>) {
            let param_count = arrow.params.items.len();
            if param_count > MAX_PARAMS {
                self.linter.add_error(
                    "max-function-params".to_string(),
                    format!(
                        "Arrow function has {} parameters (max: {}). Use an options object as the second parameter instead",
                        param_count, MAX_PARAMS
                    ),
                    arrow.span,
                );
            }
            
            oxc::ast_visit::walk::walk_arrow_function_expression(self, arrow);
        }
    }
    
    let mut visitor = MaxParamsVisitor {
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
        check_max_function_params(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.message).collect()
    }

    #[test]
    #[ignore] // Integrated into CombinedVisitor
    fn test_function_with_three_params() {
        let source = r#"
            function badFunc(a: number, b: string, c: boolean) {
                return a;
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("has 3 parameters (max: 2)"));
        assert!(errors[0].contains("badFunc"));
    }

    #[test]
    #[ignore] // Integrated into CombinedVisitor
    fn test_function_with_many_params() {
        let source = r#"
            function veryBad(a: number, b: string, c: boolean, d: any, e: string) {
                return a;
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("has 5 parameters (max: 2)"));
    }

    #[test]
    #[ignore] // Integrated into CombinedVisitor
    fn test_arrow_function_with_too_many_params() {
        let source = r#"
            const arrow = (a: number, b: string, c: boolean) => a + b;
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Arrow function has 3 parameters"));
    }

    #[test]
    fn test_function_with_two_params_ok() {
        let source = r#"
            function goodFunc(id: string, options: Options) {
                return id;
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_function_with_one_param_ok() {
        let source = r#"
            function single(value: string) {
                return value;
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_good_pattern_with_options() {
        let source = r#"
            interface CreateUserOptions {
                email: string;
                age: number;
                isAdmin: boolean;
            }
            
            function createUser(name: string, options: CreateUserOptions) {
                // Good: using options object for multiple parameters
                return { name, ...options };
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    #[ignore] // Integrated into CombinedVisitor
    fn test_method_with_too_many_params() {
        let source = r#"
            const obj = {
                method(a: number, b: string, c: boolean, d: any) {
                    return a;
                }
            };
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("has 4 parameters"));
    }
}