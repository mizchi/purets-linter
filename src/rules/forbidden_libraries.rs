use oxc_ast::ast::*;

use crate::Linter;

// Libraries that should not be used
const FORBIDDEN_LIBRARIES: &[&str] = &[
    "jquery",
    "lodash",
    "lodash/fp",
    "underscore", 
    "rxjs",
];

// Libraries with better alternatives
const PREFER_ALTERNATIVES: &[(&str, &str)] = &[
    ("minimist", "node:util parseArgs"),
    ("yargs", "node:util parseArgs"),
];

pub fn check_forbidden_libraries(linter: &mut Linter, program: &Program) {
    use oxc_ast::Visit;
    
    struct ForbiddenLibrariesVisitor<'a, 'b> {
        linter: &'a mut Linter,
        _phantom: std::marker::PhantomData<&'b ()>,
    }
    
    impl<'a, 'b> Visit<'b> for ForbiddenLibrariesVisitor<'a, 'b> {
        fn visit_import_declaration(&mut self, import: &ImportDeclaration<'b>) {
            let source = import.source.value.as_str();
            
            // Check for forbidden libraries
            if FORBIDDEN_LIBRARIES.contains(&source) || source.starts_with("lodash/") {
                self.linter.add_error(
                    "forbidden-libraries".to_string(),
                    format!("Library '{}' is forbidden. Consider using modern alternatives", source),
                    import.span,
                );
            }
            
            // Check for libraries with better alternatives
            for (lib, alternative) in PREFER_ALTERNATIVES {
                if source == *lib {
                    self.linter.add_error(
                        "forbidden-libraries".to_string(),
                        format!("Library '{}' has a better alternative. Use '{}' instead", lib, alternative),
                        import.span,
                    );
                }
            }
            
            oxc_ast::visit::walk::walk_import_declaration(self, import);
        }
        
        fn visit_call_expression(&mut self, call: &CallExpression<'b>) {
            // Check for require() calls
            if let Expression::Identifier(ident) = &call.callee {
                if ident.name == "require" && call.arguments.len() > 0 {
                    if let Argument::StringLiteral(lit) = &call.arguments[0] {
                        let source = lit.value.as_str();
                        
                        // Check for forbidden libraries in require
                        if FORBIDDEN_LIBRARIES.contains(&source) || source.starts_with("lodash/") {
                            self.linter.add_error(
                                "forbidden-libraries".to_string(),
                                format!("Library '{}' is forbidden. Consider using modern alternatives", source),
                                call.span,
                            );
                        }
                        
                        // Check for libraries with better alternatives in require
                        for (lib, alternative) in PREFER_ALTERNATIVES {
                            if source == *lib {
                                self.linter.add_error(
                                    "forbidden-libraries".to_string(),
                                    format!("Library '{}' has a better alternative. Use '{}' instead", lib, alternative),
                                    call.span,
                                );
                            }
                        }
                    }
                }
            }
            
            oxc_ast::visit::walk::walk_call_expression(self, call);
        }
    }
    
    let mut visitor = ForbiddenLibrariesVisitor {
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
        check_forbidden_libraries(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.message).collect()
    }

    #[test]
    fn test_forbidden_jquery() {
        let source = r#"
            import $ from 'jquery';
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("'jquery' is forbidden"));
    }

    #[test]
    fn test_forbidden_lodash() {
        let source = r#"
            import _ from 'lodash';
            import fp from 'lodash/fp';
            import debounce from 'lodash/debounce';
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 3);
        assert!(errors[0].contains("'lodash' is forbidden"));
        assert!(errors[1].contains("'lodash/fp' is forbidden"));
        assert!(errors[2].contains("'lodash/debounce' is forbidden"));
    }

    #[test]
    fn test_forbidden_rxjs() {
        let source = r#"
            import { Observable } from 'rxjs';
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("'rxjs' is forbidden"));
    }

    #[test]
    fn test_prefer_parseargs() {
        let source = r#"
            import minimist from 'minimist';
            import yargs from 'yargs';
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("'minimist' has a better alternative"));
        assert!(errors[0].contains("Use 'node:util parseArgs' instead"));
        assert!(errors[1].contains("'yargs' has a better alternative"));
    }

    #[test]
    fn test_forbidden_require() {
        let source = r#"
            const _ = require('lodash');
            const minimist = require('minimist');
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("'lodash' is forbidden"));
        assert!(errors[1].contains("'minimist' has a better alternative"));
    }

    #[test]
    fn test_allowed_libraries() {
        let source = r#"
            import React from 'react';
            import express from 'express';
            import axios from 'axios';
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }
}