use oxc_ast::ast::*;
use oxc_ast::Visit;

use crate::Linter;

pub fn check_no_classes(linter: &mut Linter, program: &Program) {
    struct ClassChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for ClassChecker<'a> {
        fn visit_class(&mut self, class: &Class<'a>) {
            self.linter.add_error(
                "no-classes".to_string(),
                "Classes are not allowed in pure TypeScript subset".to_string(),
                class.span,
            );
        }
    }
    
    let mut checker = ClassChecker { linter };
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
        check_no_classes(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.rule).collect()
    }

    #[test]
    fn test_class_declaration() {
        let source = r#"
            class MyClass {
                constructor() {}
            }
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-classes".to_string()));
    }

    #[test]
    fn test_class_expression() {
        let source = r#"
            const MyClass = class {
                constructor() {}
            };
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-classes".to_string()));
    }

    #[test]
    fn test_no_class() {
        let source = r#"
            function myFunction() {
                return 42;
            }
            
            const myConst = 123;
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_abstract_class() {
        let source = r#"
            abstract class AbstractClass {
                abstract method(): void;
            }
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-classes".to_string()));
    }
}
