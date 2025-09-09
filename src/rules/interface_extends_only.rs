use oxc::ast::ast::*;
use oxc::ast_visit::{Visit, walk};

use crate::Linter;

pub fn check_interface_extends_only(linter: &mut Linter, program: &Program) {
    struct InterfaceChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for InterfaceChecker<'a> {
        fn visit_ts_interface_declaration(&mut self, decl: &TSInterfaceDeclaration<'a>) {
            // Check if interface has extends clause
            if decl.extends.is_empty() {
                self.linter.add_error(
                    "interface-extends-only".to_string(),
                    format!(
                        "Interface '{}' without extends is not allowed. Use 'type' instead",
                        decl.id.name.as_str()
                    ),
                    decl.span,
                );
            }
            walk::walk_ts_interface_declaration(self, decl);
        }
    }
    
    let mut checker = InterfaceChecker { linter };
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
        let source_type = SourceType::from_path(Path::new("test.ts")).unwrap();
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut linter = Linter::new(Path::new("test-file.ts"), source, false);
        check_interface_extends_only(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.message).collect()
    }

    #[test]
    fn test_interface_without_extends() {
        let source = r#"
            interface User {
                id: string;
                name: string;
            }
        "#;
        
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("without extends is not allowed"));
    }

    #[test]
    fn test_interface_with_extends() {
        let source = r#"
            interface User {
                id: string;
            }
            
            interface Admin extends User {
                permissions: string[];
            }
        "#;
        
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1); // User interface doesn't extend anything
        assert!(errors[0].contains("Interface 'User' without extends"));
    }

    #[test]
    fn test_interface_multiple_extends() {
        let source = r#"
            interface A {}
            interface B {}
            interface C extends A, B {
                value: number;
            }
        "#;
        
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 2); // A and B don't extend anything
        assert!(errors.iter().any(|e| e.contains("Interface 'A' without extends")));
        assert!(errors.iter().any(|e| e.contains("Interface 'B' without extends")));
    }

    #[test]
    fn test_type_alias_allowed() {
        let source = r#"
            type User = {
                id: string;
                name: string;
            };
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_type_intersection() {
        let source = r#"
            type Base = { id: string };
            type Extended = Base & { name: string };
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }
}
