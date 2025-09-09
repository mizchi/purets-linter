use oxc::ast::ast::*;
use oxc::ast_visit::Visit;

use crate::Linter;

pub fn check_import_extensions(linter: &mut Linter, program: &Program) {
    struct ImportExtensionChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for ImportExtensionChecker<'a> {
        fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
            let source = import.source.value.as_str();
            
            // Check if it's a relative path import
            if source.starts_with("./") || source.starts_with("../") {
                // Check if it has .ts or .tsx extension
                if !source.ends_with(".ts") && !source.ends_with(".tsx") && !source.ends_with(".js") && !source.ends_with(".jsx") {
                    self.linter.add_error(
                        "import-extensions-required".to_string(),
                        format!("Relative imports must include .ts extension: '{}'", source),
                        import.span,
                    );
                }
            }
        }
        
        fn visit_export_named_declaration(&mut self, export: &ExportNamedDeclaration<'a>) {
            if let Some(source) = &export.source {
                let source_str = source.value.as_str();
                
                // Check if it's a relative path import
                if source_str.starts_with("./") || source_str.starts_with("../") {
                    // Check if it has .ts or .tsx extension
                    if !source_str.ends_with(".ts") && !source_str.ends_with(".tsx") && !source_str.ends_with(".js") && !source_str.ends_with(".jsx") {
                        self.linter.add_error(
                            "import-extensions-required".to_string(),
                            format!("Relative imports must include .ts extension: '{}'", source_str),
                            export.span,
                        );
                    }
                }
            }
        }
        
        fn visit_export_all_declaration(&mut self, export: &ExportAllDeclaration<'a>) {
            let source = export.source.value.as_str();
            
            // Check if it's a relative path import
            if source.starts_with("./") || source.starts_with("../") {
                // Check if it has .ts or .tsx extension
                if !source.ends_with(".ts") && !source.ends_with(".tsx") && !source.ends_with(".js") && !source.ends_with(".jsx") {
                    self.linter.add_error(
                        "import-extensions-required".to_string(),
                        format!("Relative imports must include .ts extension: '{}'", source),
                        export.span,
                    );
                }
            }
        }
    }
    
    let mut checker = ImportExtensionChecker { linter };
    checker.visit_program(program);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Linter;
    use oxc::allocator::Allocator;
    use oxc::parser::{Parser, ParserReturn};
    use oxc::span::SourceType;
    use std::path::Path;

    fn run_test_with_code(source_text: &str, expected_error_count: usize, expected_messages: &[&str]) {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_import_extensions(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), expected_error_count);
        for message in expected_messages {
            assert!(errors.iter().any(|e| e.message.contains(message)));
        }
    }

    #[test]
    fn test_relative_imports_without_extension() {
        let source_text = r#"
import { foo } from './utils';
import { bar } from '../lib/helper';
"#;
        run_test_with_code(source_text, 2, &["./utils", "../lib/helper"]);
    }

    #[test]
    fn test_relative_imports_with_extension() {
        let source_text = r#"
import { baz } from './other.ts';
import { qux } from '../shared/common.ts';
"#;
        run_test_with_code(source_text, 0, &[]);
    }

    #[test]
    fn test_non_relative_imports() {
        let source_text = r#"
import { createElement } from 'react';
"#;
        run_test_with_code(source_text, 0, &[]);
    }

    #[test]
    fn test_re_export_without_extension() {
        let source_text = r#"
export { something } from './another';
"#;
        run_test_with_code(source_text, 1, &["./another"]);
    }

    #[test]
    fn test_mixed_imports() {
        let source_text = r#"
import { foo } from './utils';
import { bar } from '../lib/helper';
import { baz } from './other.ts';
import { qux } from '../shared/common.ts';
import { createElement } from 'react';
export { something } from './another';
"#;
        run_test_with_code(source_text, 3, &["./utils", "../lib/helper", "./another"]);
    }
}
