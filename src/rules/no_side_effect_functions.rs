use oxc::ast::ast::*;

use crate::Linter;

// Functions that have side effects and should not be called directly
const SIDE_EFFECT_FUNCTIONS: &[(&str, &str)] = &[("Math", "random"), ("Date", "now")];

const SIDE_EFFECT_GLOBAL_FUNCTIONS: &[&str] = &[
    "setTimeout",
    "setInterval",
    "setImmediate",
    "requestAnimationFrame",
    "requestIdleCallback",
];

pub fn check_no_side_effect_functions(linter: &mut Linter, program: &Program) {
    use oxc::ast_visit::Visit;

    struct SideEffectVisitor<'a, 'b> {
        linter: &'a mut Linter,
        in_function: bool,
        in_default_parameter: bool,
        _phantom: std::marker::PhantomData<&'b ()>,
    }

    impl<'a, 'b> Visit<'b> for SideEffectVisitor<'a, 'b> {
        fn visit_function(&mut self, func: &Function<'b>, _: oxc::syntax::scope::ScopeFlags) {
            let was_in_function = self.in_function;
            self.in_function = true;

            // Visit parameters to check for default values
            for param in &func.params.items {
                if param.pattern.type_annotation.is_some() {
                    // Check default parameter values
                    self.in_default_parameter = true;
                    oxc::ast_visit::walk::walk_formal_parameter(self, param);
                    self.in_default_parameter = false;
                }
            }

            // Visit function body
            if let Some(body) = &func.body {
                self.visit_function_body(body);
            }

            self.in_function = was_in_function;
        }

        fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'b>) {
            let was_in_function = self.in_function;
            self.in_function = true;

            // Visit parameters
            for param in &arrow.params.items {
                self.in_default_parameter = true;
                oxc::ast_visit::walk::walk_formal_parameter(self, param);
                self.in_default_parameter = false;
            }

            // Visit body
            oxc::ast_visit::walk::walk_arrow_function_expression(self, arrow);

            self.in_function = was_in_function;
        }

        fn visit_new_expression(&mut self, new_expr: &NewExpression<'b>) {
            // Check for new Date()
            if self.in_function && !self.in_default_parameter {
                if let Expression::Identifier(ident) = &new_expr.callee {
                    if ident.name == "Date" {
                        self.linter.add_error(
                            "no-side-effect-functions".to_string(),
                            "Direct use of 'new Date()' is not allowed in functions. Pass it as a parameter or use a default parameter instead".to_string(),
                            new_expr.span,
                        );
                    }
                }
            }

            oxc::ast_visit::walk::walk_new_expression(self, new_expr);
        }

        fn visit_call_expression(&mut self, call: &CallExpression<'b>) {
            if self.in_function && !self.in_default_parameter {
                // Check for Math.random(), Date.now()
                if let Some(member) = call.callee.as_member_expression() {
                    if let MemberExpression::StaticMemberExpression(static_member) = &member {
                        if let Expression::Identifier(obj) = &static_member.object {
                            let obj_name = obj.name.as_str();
                            let method_name = static_member.property.name.as_str();

                            for (object, method) in SIDE_EFFECT_FUNCTIONS {
                                if obj_name == *object && method_name == *method {
                                    self.linter.add_error(
                                        "no-side-effect-functions".to_string(),
                                        format!(
                                            "Direct use of '{}.{}()' is not allowed in functions. Pass it as a parameter or use a default parameter instead",
                                            object, method
                                        ),
                                        call.span,
                                    );
                                }
                            }
                        }
                    }
                }

                // Check for global side-effect functions
                if let Expression::Identifier(ident) = &call.callee {
                    if SIDE_EFFECT_GLOBAL_FUNCTIONS.contains(&ident.name.as_str()) {
                        self.linter.add_error(
                            "no-side-effect-functions".to_string(),
                            format!(
                                "Direct use of '{}()' is not allowed in functions. Pass it as a parameter or use a default parameter instead",
                                ident.name
                            ),
                            call.span,
                        );
                    }
                }
            }

            oxc::ast_visit::walk::walk_call_expression(self, call);
        }
    }

    let mut visitor = SideEffectVisitor {
        linter,
        in_function: false,
        in_default_parameter: false,
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
        check_no_side_effect_functions(&mut linter, &ret.program);

        linter.errors.into_iter().map(|e| e.message).collect()
    }

    #[test]
    fn test_math_random_direct() {
        let source = r#"
            function getRandom() {
                return Math.random();
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Math.random()"));
    }

    #[test]
    fn test_math_random_as_default_param() {
        let source = r#"
            function getRandom(randomFn = () => Math.random()) {
                return randomFn();
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_date_now_direct() {
        let source = r#"
            function getTimestamp() {
                return Date.now();
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Date.now()"));
    }

    #[test]
    fn test_new_date_direct() {
        let source = r#"
            function getCurrentDate() {
                return new Date();
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("new Date()"));
    }

    #[test]
    fn test_settimeout_direct() {
        let source = r#"
            function delayedAction() {
                setTimeout(() => console.log("hello"), 1000);
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("setTimeout()"));
    }

    #[test]
    fn test_settimeout_as_param() {
        let source = r#"
            function delayedAction(scheduler = setTimeout) {
                scheduler(() => console.log("hello"), 1000);
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_arrow_function() {
        let source = r#"
            const getRandom = () => {
                return Math.random();
            };
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Math.random()"));
    }

    #[test]
    fn test_outside_function_allowed() {
        let source = r#"
            const timestamp = Date.now();
            const random = Math.random();
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }
}
