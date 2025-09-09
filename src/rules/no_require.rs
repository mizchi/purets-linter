use oxc_ast::ast::*;

use crate::Linter;

pub fn check_no_require(linter: &mut Linter, program: &Program) {
    use oxc_ast::Visit;
    
    struct NoRequireVisitor<'a, 'b> {
        linter: &'a mut Linter,
        _phantom: std::marker::PhantomData<&'b ()>,
    }
    
    impl<'a, 'b> Visit<'b> for NoRequireVisitor<'a, 'b> {
        fn visit_call_expression(&mut self, call: &CallExpression<'b>) {
            // Check for require() calls
            if let Expression::Identifier(ident) = &call.callee {
                if ident.name == "require" {
                    self.linter.add_error(
                        "no-require".to_string(),
                        "require() is not allowed. Use ES6 import statements instead".to_string(),
                        call.span,
                    );
                }
            }
            
            oxc_ast::visit::walk::walk_call_expression(self, call);
        }
    }
    
    let mut visitor = NoRequireVisitor {
        linter,
        _phantom: std::marker::PhantomData,
    };
    visitor.visit_program(program);
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
        
        let mut linter = Linter::new(Path::new("test.ts"), source, false);
        check_no_require(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.message).collect()
    }

    #[test]
    fn test_require_call() {
        let source = r#"
            const fs = require('fs');
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("require() is not allowed"));
    }

    #[test]
    fn test_dynamic_require() {
        let source = r#"
            const module = require('./module');
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("ES6 import"));
    }

    #[test]
    fn test_import_allowed() {
        let source = r#"
            import fs from 'fs';
            import { readFile } from 'fs/promises';
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }
}