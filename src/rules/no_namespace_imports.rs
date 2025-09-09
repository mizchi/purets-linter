use oxc_ast::ast::*;

use crate::Linter;

pub fn check_no_namespace_imports(linter: &mut Linter, program: &Program) {
    for item in &program.body {
        if let Statement::ImportDeclaration(import) = item {
            if let Some(specifiers) = &import.specifiers {
                for specifier in specifiers {
                    if matches!(
                        specifier,
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)
                    ) {
                        linter.add_error(
                            "no-namespace-imports".to_string(),
                            format!("Namespace imports from '{}' are not allowed. Use named imports instead", 
                                import.source.value),
                            import.span,
                        );
                    }
                }
            }
        }
    }
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
        check_no_namespace_imports(&mut linter, &ret.program);

        linter.errors.into_iter().map(|e| e.rule).collect()
    }

    #[test]
    fn test_namespace_import() {
        let source = r#"
            import * as utils from './utils.ts';
        "#;

        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-namespace-imports".to_string()));
    }

    #[test]
    fn test_named_imports_allowed() {
        let source = r#"
            import { foo, bar } from './module.ts';
            import type { MyType } from './types.ts';
        "#;

        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_default_import_allowed() {
        let source = r#"
            import React from 'react';
            import myModule from './module.ts';
        "#;

        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_mixed_with_namespace() {
        let source = r#"
            import defaultExport, * as namespace from './module.ts';
        "#;

        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-namespace-imports".to_string()));
    }
}
