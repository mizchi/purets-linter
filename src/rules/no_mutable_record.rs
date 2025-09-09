use crate::Linter;
use oxc::ast::ast::*;
use oxc::ast_visit::Visit;

pub fn check_no_mutable_record(linter: &mut Linter, program: &Program) {
    struct RecordChecker<'a> {
        linter: &'a mut Linter,
    }

    impl<'a> Visit<'a> for RecordChecker<'a> {
        fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
            // Check if type annotation is Record<K, V>
            if let Some(type_ann) = &decl.id.type_annotation {
                if let TSType::TSTypeReference(type_ref) = &type_ann.type_annotation {
                    if let TSTypeName::IdentifierReference(id) = &type_ref.type_name {
                        if id.name == "Record" {
                            // Check if initialized with empty object
                            if let Some(Expression::ObjectExpression(obj)) = &decl.init {
                                if obj.properties.is_empty() {
                                    self.linter.add_error(
                                        "no-mutable-record".to_string(),
                                        "Mutable Record<K, V> = {} is not allowed. Use Map instead for mutable key-value collections".to_string(),
                                        decl.span,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut checker = RecordChecker { linter };
    checker.visit_program(program);
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
        let source_type = SourceType::from_path("test.ts").unwrap();
        let ret = Parser::new(&allocator, source, source_type).parse();

        let mut linter = Linter::new(Path::new("test.ts"), source, false);
        check_no_mutable_record(&mut linter, &ret.program);

        linter.errors.into_iter().map(|e| e.rule).collect()
    }

    #[test]
    fn test_empty_record() {
        let source = r#"
            const obj: Record<string, number> = {};
        "#;

        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors.contains(&"no-mutable-record".to_string()));
    }

    #[test]
    fn test_record_with_initial_values() {
        let source = r#"
            const obj: Record<string, number> = { a: 1, b: 2 };
        "#;

        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_map_allowed() {
        let source = r#"
            const map = new Map<string, number>();
        "#;

        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_plain_object() {
        let source = r#"
            const obj = {};
        "#;

        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_typed_object() {
        let source = r#"
            const obj: { [key: string]: number } = {};
        "#;

        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }
}
