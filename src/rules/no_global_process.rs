use oxc::ast::ast::*;

use crate::Linter;

pub fn check_no_global_process(linter: &mut Linter, program: &Program) {
    use oxc::ast_visit::Visit;
    use std::collections::HashSet;
    
    struct NoGlobalProcessVisitor<'a, 'b> {
        linter: &'a mut Linter,
        // Track if process is imported from 'node:process'
        process_imported: bool,
        // Track imported names
        imported_names: HashSet<String>,
        _phantom: std::marker::PhantomData<&'b ()>,
    }
    
    impl<'a, 'b> NoGlobalProcessVisitor<'a, 'b> {
        fn new(linter: &'a mut Linter) -> Self {
            Self {
                linter,
                process_imported: false,
                imported_names: HashSet::new(),
                _phantom: std::marker::PhantomData,
            }
        }
        
        fn check_imports(&mut self, program: &Program<'b>) {
            for item in &program.body {
                if let Statement::ImportDeclaration(import) = item {
                    // Check if importing from 'node:process'
                    if import.source.value == "node:process" || import.source.value == "process" {
                        self.process_imported = true;
                        
                        // Track what's imported
                        if let Some(specifiers) = &import.specifiers {
                            for spec in specifiers {
                                match spec {
                                    ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                                        self.imported_names.insert(default.local.name.to_string());
                                    }
                                    ImportDeclarationSpecifier::ImportSpecifier(named) => {
                                        self.imported_names.insert(named.local.name.to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    impl<'a, 'b> Visit<'b> for NoGlobalProcessVisitor<'a, 'b> {
        fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'b>) {
            // Check for global process usage
            if ident.name == "process" && !self.imported_names.contains("process") {
                self.linter.add_error(
                    "no-global-process".to_string(),
                    "Global 'process' is not allowed. Import it from 'node:process' instead".to_string(),
                    ident.span,
                );
            }
            
            oxc::ast_visit::walk::walk_identifier_reference(self, ident);
        }
    }
    
    let mut visitor = NoGlobalProcessVisitor::new(linter);
    visitor.check_imports(program);
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
        check_no_global_process(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.message).collect()
    }

    #[test]
    fn test_global_process() {
        let source = r#"
            const env = process.env.NODE_ENV;
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Global 'process' is not allowed"));
    }

    #[test]
    fn test_process_exit() {
        let source = r#"
            if (error) {
                process.exit(1);
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Import it from 'node:process'"));
    }

    #[test]
    fn test_imported_process_ok() {
        let source = r#"
            import process from 'node:process';
            const env = process.env.NODE_ENV;
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_imported_from_process_ok() {
        let source = r#"
            import process from 'process';
            process.exit(0);
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_destructured_import() {
        let source = r#"
            import { env, exit } from 'node:process';
            console.log(env.HOME);
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }
}