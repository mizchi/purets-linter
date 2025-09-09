use oxc_ast::ast::*;
use oxc_span::Span;

use crate::Linter;

/// Check path-based restrictions for TypeScript files
/// 
/// Rules:
/// - io/**/*.ts: Only async functions are allowed
/// - pure/**/*.ts: Pure functions only, export function name must match filename
/// - types/**/*.ts: Only one type export allowed, must match filename
/// - *_test.ts: Must import function with same name (minus _test suffix)
pub fn check_path_based_restrictions(
    linter: &mut Linter,
    program: &Program,
    file_path: &str,
) {
    let normalized_path = file_path.replace('\\', "/");
    
    // Check test files first (they can be in any directory)
    if normalized_path.ends_with("_test.ts") {
        check_test_file_imports(linter, program, &normalized_path);
        return; // Test files don't need to follow other path restrictions
    }
    
    // Check io/**/*.ts - only async functions allowed
    if normalized_path.contains("/io/") && normalized_path.ends_with(".ts") {
        check_io_async_only(linter, program);
    }
    
    // Check pure/**/*.ts - pure functions with filename match
    if normalized_path.contains("/pure/") && normalized_path.ends_with(".ts") {
        check_pure_functions(linter, program, &normalized_path);
    }
    
    // Check types/**/*.ts - single type export matching filename
    if normalized_path.contains("/types/") && normalized_path.ends_with(".ts") {
        check_type_definitions(linter, program, &normalized_path);
    }
}

/// Check that io/**/*.ts files only contain async functions
fn check_io_async_only(linter: &mut Linter, program: &Program) {
    for stmt in &program.body {
        match stmt {
            Statement::ExportNamedDeclaration(export) => {
                if let Some(Declaration::FunctionDeclaration(func)) = &export.declaration {
                    if !func.r#async {
                        linter.add_error(
                            "path-based-restrictions".to_string(),
                            "Functions in io/**/*.ts must be async".to_string(),
                            func.span,
                        );
                    }
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                match &export.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                        if !func.r#async {
                            linter.add_error(
                                "path-based-restrictions".to_string(),
                                "Functions in io/**/*.ts must be async".to_string(),
                                func.span,
                            );
                        }
                    }
                    _ => {
                        // Handle other expression types if needed
                    }
                }
            }
            Statement::FunctionDeclaration(func) => {
                // Non-exported functions in io files should also be async
                if !func.r#async {
                    linter.add_error(
                        "path-based-restrictions".to_string(),
                        "Functions in io/**/*.ts must be async".to_string(),
                        func.span,
                    );
                }
            }
            _ => {}
        }
    }
}

/// Check that pure/**/*.ts files contain pure functions with filename match
fn check_pure_functions(linter: &mut Linter, program: &Program, file_path: &str) {
    // Extract filename without extension
    let filename = file_path
        .rsplit('/')
        .next()
        .unwrap_or("")
        .trim_end_matches(".ts");
    
    let mut found_matching_export = false;
    let mut export_count = 0;
    
    // First, check that pure files don't import from io
    for stmt in &program.body {
        if let Statement::ImportDeclaration(import) = stmt {
            let source = import.source.value.as_str();
            if source.contains("/io/") {
                linter.add_error(
                    "path-based-restrictions".to_string(),
                    "pure/**/*.ts files cannot import from io/**/*.ts (pure functions cannot depend on I/O)".to_string(),
                    import.span,
                );
            }
        }
    }
    
    for stmt in &program.body {
        match stmt {
            Statement::ExportNamedDeclaration(export) => {
                if let Some(Declaration::FunctionDeclaration(func)) = &export.declaration {
                    export_count += 1;
                    
                    // Check if function is async (not allowed in pure)
                    if func.r#async {
                        linter.add_error(
                            "path-based-restrictions".to_string(),
                            "Functions in pure/**/*.ts cannot be async".to_string(),
                            func.span,
                        );
                    }
                    
                    // Check if function name matches filename
                    if let Some(id) = &func.id {
                        if id.name.as_str() == filename {
                            found_matching_export = true;
                        }
                    }
                }
            }
            Statement::FunctionDeclaration(func) => {
                // Non-exported functions in pure files should also not be async
                if func.r#async {
                    linter.add_error(
                        "path-based-restrictions".to_string(),
                        "Functions in pure/**/*.ts cannot be async".to_string(),
                        func.span,
                    );
                }
            }
            _ => {}
        }
    }
    
    // Check if we found a function matching the filename
    if export_count > 0 && !found_matching_export {
        linter.add_error(
            "path-based-restrictions".to_string(),
            format!("pure/**/*.ts must export a function named '{}' matching the filename", filename),
            Span::new(0, 0),
        );
    }
}

/// Check that types/**/*.ts files contain only one type export matching filename
fn check_type_definitions(linter: &mut Linter, program: &Program, file_path: &str) {
    // Extract filename without extension
    let filename = file_path
        .rsplit('/')
        .next()
        .unwrap_or("")
        .trim_end_matches(".ts");
    
    let mut type_exports = Vec::new();
    let mut found_matching_type = false;
    
    for stmt in &program.body {
        match stmt {
            Statement::ExportNamedDeclaration(export) => {
                // Check for type alias exports
                if let Some(Declaration::TSTypeAliasDeclaration(type_alias)) = &export.declaration {
                    let name = type_alias.id.name.as_str();
                    type_exports.push((name, type_alias.span));
                    if name == filename {
                        found_matching_type = true;
                    }
                }
                
                // Check for interface exports
                if let Some(Declaration::TSInterfaceDeclaration(interface)) = &export.declaration {
                    let name = interface.id.name.as_str();
                    type_exports.push((name, interface.span));
                    if name == filename {
                        found_matching_type = true;
                    }
                }
                
                // Check for enum exports (should be discouraged in types)
                if let Some(Declaration::TSEnumDeclaration(enum_decl)) = &export.declaration {
                    linter.add_error(
                        "path-based-restrictions".to_string(),
                        "types/**/*.ts should only export type definitions, not enums".to_string(),
                        enum_decl.span,
                    );
                }
                
                // Check for function/class exports (not allowed in types)
                if let Some(decl) = &export.declaration {
                    match decl {
                        Declaration::FunctionDeclaration(func) => {
                            linter.add_error(
                                "path-based-restrictions".to_string(),
                                "types/**/*.ts should only export type definitions, not functions".to_string(),
                                func.span,
                            );
                        }
                        Declaration::ClassDeclaration(class) => {
                            linter.add_error(
                                "path-based-restrictions".to_string(),
                                "types/**/*.ts should only export type definitions, not classes".to_string(),
                                class.span,
                            );
                        }
                        Declaration::VariableDeclaration(var) => {
                            linter.add_error(
                                "path-based-restrictions".to_string(),
                                "types/**/*.ts should only export type definitions, not variables".to_string(),
                                var.span,
                            );
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    
    // Check if we have exactly one type export
    if type_exports.len() > 1 {
        for (name, span) in &type_exports {
            if *name != filename {
                linter.add_error(
                    "path-based-restrictions".to_string(),
                    format!("types/**/*.ts should only export one type named '{}' matching the filename", filename),
                    *span,
                );
            }
        }
    } else if type_exports.len() == 1 && !found_matching_type {
        linter.add_error(
            "path-based-restrictions".to_string(),
            format!("Type export must be named '{}' to match the filename", filename),
            type_exports[0].1,
        );
    }
}

/// Check that *_test.ts files import the function with matching name
fn check_test_file_imports(linter: &mut Linter, program: &Program, file_path: &str) {
    // Extract base filename without _test.ts suffix
    let filename = file_path
        .rsplit('/')
        .next()
        .unwrap_or("")
        .trim_end_matches("_test.ts");
    
    if filename.is_empty() {
        return;
    }
    
    let mut found_matching_import = false;
    let mut has_imports = false;
    
    // Check import statements
    for stmt in &program.body {
        if let Statement::ImportDeclaration(import) = stmt {
            has_imports = true;
            
            // Check if any specifier imports the expected function name
            if let Some(specifiers) = &import.specifiers {
                for specifier in specifiers {
                    match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                            let imported = spec.imported.name();
                            if imported == filename {
                                found_matching_import = true;
                                break;
                            }
                        }
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => {
                            // For default imports, check if the source matches
                            let source = import.source.value.as_str();
                            if source.contains(filename) || source.ends_with(&format!("/{}.ts", filename)) {
                                found_matching_import = true;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
            
            if found_matching_import {
                break;
            }
        }
    }
    
    // Report error if the matching import was not found
    if has_imports && !found_matching_import {
        linter.add_error(
            "path-based-restrictions".to_string(),
            format!("Test file '{}' must import function '{}' from the module being tested", 
                    file_path.rsplit('/').next().unwrap_or(""), filename),
            Span::new(0, 0),
        );
    } else if !has_imports {
        linter.add_error(
            "path-based-restrictions".to_string(),
            format!("Test file '{}' must have at least one import statement", 
                    file_path.rsplit('/').next().unwrap_or("")),
            Span::new(0, 0),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    fn parse_and_check(source: &str, file_path: &str) -> Vec<String> {
        use std::path::Path;
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(file_path).unwrap();
        let parser_ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut linter = Linter::new(Path::new(file_path), source, false);
        check_path_based_restrictions(&mut linter, &parser_ret.program, file_path);
        
        linter
            .errors
            .into_iter()
            .map(|e| e.message)
            .collect()
    }

    #[test]
    fn test_io_async_functions() {
        // Non-async function in io/ should error
        let source = r#"
            export function readFile(path: string): string {
                return "content";
            }
        "#;
        let errors = parse_and_check(source, "src/io/file.ts");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("must be async"));

        // Async function in io/ should pass
        let source = r#"
            export async function readFile(path: string): Promise<string> {
                return "content";
            }
        "#;
        let errors = parse_and_check(source, "src/io/file.ts");
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_pure_functions() {
        // Async function in pure/ should error
        let source = r#"
            export async function calculate(a: number): Promise<number> {
                return a * 2;
            }
        "#;
        let errors = parse_and_check(source, "src/pure/calculate.ts");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("cannot be async"));

        // Function name not matching filename should error
        let source = r#"
            export function wrongName(a: number): number {
                return a * 2;
            }
        "#;
        let errors = parse_and_check(source, "src/pure/calculate.ts");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("must export a function named 'calculate'"));

        // Pure function importing from io should error
        let source = r#"
            import { readFile } from "../io/file";
            
            export function calculate(a: number): number {
                return a * 2;
            }
        "#;
        let errors = parse_and_check(source, "src/pure/calculate.ts");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("cannot import from io"));

        // Correct pure function should pass
        let source = r#"
            export function calculate(a: number): number {
                return a * 2;
            }
        "#;
        let errors = parse_and_check(source, "src/pure/calculate.ts");
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_type_definitions() {
        // Multiple type exports should error
        let source = r#"
            export type Point = { x: number; y: number };
            export type Vector = { dx: number; dy: number };
        "#;
        let errors = parse_and_check(source, "src/types/Point.ts");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("should only export one type"));

        // Type name not matching filename should error
        let source = r#"
            export type Vector = { x: number; y: number };
        "#;
        let errors = parse_and_check(source, "src/types/Point.ts");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("must be named 'Point'"));

        // Function export in types should error
        let source = r#"
            export function createPoint(): Point {
                return { x: 0, y: 0 };
            }
            type Point = { x: number; y: number };
        "#;
        let errors = parse_and_check(source, "src/types/Point.ts");
        assert!(errors.iter().any(|e| e.contains("not functions")));

        // Correct type export should pass
        let source = r#"
            export type Point = { x: number; y: number };
        "#;
        let errors = parse_and_check(source, "src/types/Point.ts");
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_test_file_imports() {
        // Test file without matching import should error
        let source = r#"
            import { otherFunction } from "./other";
            
            describe("add", () => {
                it("should work", () => {});
            });
        "#;
        let errors = parse_and_check(source, "src/add_test.ts");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("must import function 'add'"));

        // Test file with no imports should error
        let source = r#"
            describe("add", () => {
                it("should work", () => {});
            });
        "#;
        let errors = parse_and_check(source, "src/add_test.ts");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("must have at least one import"));

        // Test file with matching import should pass
        let source = r#"
            import { add } from "./add";
            
            describe("add", () => {
                it("should add two numbers", () => {
                    expect(add(1, 2)).toBe(3);
                });
            });
        "#;
        let errors = parse_and_check(source, "src/add_test.ts");
        assert_eq!(errors.len(), 0);

        // Test file with default import matching should pass
        let source = r#"
            import calculate from "./calculate.ts";
            
            describe("calculate", () => {
                it("should work", () => {});
            });
        "#;
        let errors = parse_and_check(source, "src/calculate_test.ts");
        assert_eq!(errors.len(), 0);
    }
}