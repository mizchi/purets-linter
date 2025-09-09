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
    use crate::Linter;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;
    use std::path::Path;

    fn parse_and_check(source: &str) -> Vec<String> {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path("test.ts").unwrap();
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut linter = Linter::new(Path::new("test.ts"), source, false);
        check_empty_array_requires_type(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.rule).collect()
    }

    #[test]
    fn test_empty_array_without_type() {
        let source = r#"
            const array = [];
        "#;
        
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors.contains(&"empty-array-requires-type".to_string()));
    }

    #[test]
    fn test_empty_array_with_type() {
        let source = r#"
            const array: Array<number> = [];
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_empty_array_with_type_literal() {
        let source = r#"
            const array: number[] = [];
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_non_empty_array() {
        let source = r#"
            const array = [1, 2, 3];
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_let_empty_array_without_type() {
        let source = r#"
            let array = [];
        "#;
        
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors.contains(&"empty-array-requires-type".to_string()));
    }

    #[test]
    fn test_const_assertion() {
        let source = r#"
            const array = [] as const;
        "#;
        
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1); // Still requires explicit type
    }
}