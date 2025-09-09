use oxc_ast::ast::*;

use crate::Linter;

pub fn check_no_reexports(linter: &mut Linter, program: &Program) {
    // Allow re-exports in index.ts and entry point files
    let filename = linter.path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    
    // Check if this is an entry point file
    let is_entry_point = filename == "index" || 
                        linter.is_entry_point || 
                        linter.is_main_entry;
    
    if linter.verbose {
        eprintln!("DEBUG no_reexports: filename={}, is_entry_point={}, is_entry={}, is_main={}", 
                 filename, is_entry_point, linter.is_entry_point, linter.is_main_entry);
    }
    
    if is_entry_point {
        // For entry points, validate re-export rules:
        // 1. Must use named exports: export { name } from "..."
        // 2. Cannot use namespace exports: export * from "..." or export * as ns from "..."
        for item in &program.body {
            match item {
                Statement::ExportAllDeclaration(export) => {
                    linter.add_error(
                        "no-reexports".to_string(),
                        format!("Namespace re-exports are not allowed in entry points. Use named exports: export {{ name }} from '{}'", export.source.value),
                        export.span,
                    );
                }
                Statement::ExportNamedDeclaration(export) => {
                    // Named re-exports are allowed in entry points
                    // Additional validation could be added here for:
                    // - Function name matching
                    // - No leading underscore
                    // - No @internal JSDoc tag
                }
                _ => {}
            }
        }
    } else {
        // For non-entry files, no re-exports are allowed
        for item in &program.body {
            match item {
                Statement::ExportAllDeclaration(export) => {
                    linter.add_error(
                        "no-reexports".to_string(),
                        format!("Re-exports from '{}' are not allowed", export.source.value),
                        export.span,
                    );
                }
                Statement::ExportNamedDeclaration(export) => {
                    if export.source.is_some() && !export.specifiers.is_empty() {
                        linter.add_error(
                            "no-reexports".to_string(),
                            format!("Re-exports from '{}' are not allowed", 
                                export.source.as_ref().unwrap().value),
                            export.span,
                        );
                    }
                }
                _ => {}
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
        check_no_reexports(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.rule).collect()
    }

    #[test]
    fn test_export_all() {
        let source = r#"
            export * from './other.ts';
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-reexports".to_string()));
    }

    #[test]
    fn test_named_reexport() {
        let source = r#"
            export { foo, bar } from './module.ts';
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-reexports".to_string()));
    }

    #[test]
    fn test_renamed_reexport() {
        let source = r#"
            export { foo as bar } from './module.ts';
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-reexports".to_string()));
    }

    #[test]
    fn test_normal_exports_allowed() {
        let source = r#"
            const foo = 42;
            export { foo };
            
            export function bar() {
                return 'bar';
            }
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_export_namespace() {
        let source = r#"
            export * as utils from './utils.ts';
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-reexports".to_string()));
    }
}
