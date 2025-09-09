use oxc_ast::ast::*;
use oxc_ast::Visit;
use crate::Linter;

pub fn check_empty_array_requires_type(linter: &mut Linter, program: &Program) {
    struct ArrayChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for ArrayChecker<'a> {
        fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
            // Check if initializer is an empty array
            if let Some(Expression::ArrayExpression(array)) = &decl.init {
                if array.elements.is_empty() {
                    // Check if type annotation exists
                    if decl.id.type_annotation.is_none() {
                        if let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind {
                            self.linter.add_error(
                                "empty-array-requires-type".to_string(),
                                format!("Empty array '{}' requires type annotation (e.g., const {}: Array<number> = [])", id.name, id.name),
                                decl.span,
                            );
                        }
                    }
                }
            }
        }
    }
    
    let mut checker = ArrayChecker { linter };
    checker.visit_program(program);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test::*;

    fn check(source: &str) -> Vec<String> {
        check_rule(source, check_empty_array_requires_type)
    }

    #[test]
    fn test_empty_array_without_type() {
        let source = r#"
            const array = [];
        "#;
        
        let errors = check(source);
        assert_eq!(errors.len(), 1);
        assert_errors_contain(&errors, &["Empty array 'array' requires type annotation"]);
    }

    #[test]
    fn test_empty_array_with_type() {
        let source = r#"
            const array: Array<number> = [];
        "#;
        
        let errors = check(source);
        assert_no_errors(&errors);
    }

    #[test]
    fn test_empty_array_with_type_literal() {
        let source = r#"
            const array: number[] = [];
        "#;
        
        let errors = check(source);
        assert_no_errors(&errors);
    }

    #[test]
    fn test_non_empty_array() {
        let source = r#"
            const array = [1, 2, 3];
        "#;
        
        let errors = check(source);
        assert_no_errors(&errors);
    }

    #[test]
    fn test_let_empty_array_without_type() {
        let source = r#"
            let array = [];
        "#;
        
        let errors = check(source);
        assert_eq!(errors.len(), 1);
        assert_errors_contain(&errors, &["Empty array 'array' requires type annotation"]);
    }

    #[test]
    fn test_const_assertion() {
        let source = r#"
            const array = [] as const;
        "#;
        
        let errors = check(source);
        assert_no_errors(&errors); // TODO: const assertion handling needs review
    }
}