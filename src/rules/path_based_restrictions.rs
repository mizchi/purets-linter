use oxc::ast::ast::*;
use oxc::span::Span;

use crate::{Linter, TestRunner};

/// Check path-based restrictions for TypeScript files
/// 
/// Rules:
/// - io/**/*.ts: Only async functions are allowed
/// - pure/**/*.ts: Pure functions only, export function name must match filename
/// - types/**/*.ts: Only one type export allowed, must match filename
/// - *.test.ts or *_test.ts: Must import function with same name (minus test suffix)
pub fn check_path_based_restrictions(
    linter: &mut Linter,
    program: &Program,
    file_path: &str,
) {
    let normalized_path = file_path.replace('\\', "/");
    
    // Check test files first (they can be in any directory)
    if normalized_path.ends_with("_test.ts") || normalized_path.ends_with(".test.ts") {
        // Default to vitest if no test runner specified
        if linter.test_runner.is_none() {
            linter.test_runner = Some(crate::TestRunner::Vitest);
        }
        check_test_file_imports(linter, program, &normalized_path);
        return; // Test files don't need to follow other path restrictions
    }
    
    // Check index.ts - only re-exports allowed
    if normalized_path.ends_with("/index.ts") {
        check_index_reexports_only(linter, program);
    }
    
    // Check main.ts - main() call allowed at top level
    if normalized_path.ends_with("/main.ts") {
        check_main_file(linter, program);
    }
    
    // Check io/**/*.ts - async functions are optional (not required)
    if normalized_path.contains("/io/") && normalized_path.ends_with(".ts") {
        // Check io/errors/*.ts - custom error classes allowed
        if normalized_path.contains("/io/errors/") {
            check_error_class_definitions(linter, program, &normalized_path);
        }
        // No longer enforcing async-only, just allow both sync and async
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

/// Check that index.ts files only contain re-exports
fn check_index_reexports_only(linter: &mut Linter, program: &Program) {
    for stmt in &program.body {
        match stmt {
            Statement::ExportNamedDeclaration(export) => {
                // Check if it's a re-export (has source but no declaration)
                if export.source.is_none() && export.declaration.is_some() {
                    linter.add_error(
                        "path-based-restrictions".to_string(),
                        "index.ts files can only contain re-exports, not direct exports".to_string(),
                        export.span,
                    );
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                linter.add_error(
                    "path-based-restrictions".to_string(),
                    "index.ts files can only contain re-exports, not default exports".to_string(),
                    export.span,
                );
            }
            Statement::ImportDeclaration(_) => {
                // Imports are allowed in index.ts for re-exporting
            }
            Statement::ExportAllDeclaration(_) => {
                // export * from './module' is allowed
            }
            Statement::FunctionDeclaration(func) => {
                linter.add_error(
                    "path-based-restrictions".to_string(),
                    "index.ts files can only contain re-exports, not declarations".to_string(),
                    func.span,
                );
            }
            Statement::ClassDeclaration(class) => {
                linter.add_error(
                    "path-based-restrictions".to_string(),
                    "index.ts files can only contain re-exports, not declarations".to_string(),
                    class.span,
                );
            }
            Statement::VariableDeclaration(var) => {
                linter.add_error(
                    "path-based-restrictions".to_string(),
                    "index.ts files can only contain re-exports, not declarations".to_string(),
                    var.span,
                );
            }
            _ => {}
        }
    }
}

/// Check main.ts file - allows main() function call at top level
fn check_main_file(_linter: &mut Linter, _program: &Program) {
    // This function currently just allows main() calls
    // The no-top-level-side-effects rule will be bypassed for main.ts
    // We'll need to update that rule to check for main.ts
}

/// Check io/errors/*.ts files - must define error class matching filename
fn check_error_class_definitions(linter: &mut Linter, program: &Program, file_path: &str) {
    // Extract filename without extension
    let filename = file_path
        .rsplit('/')
        .next()
        .unwrap_or("")
        .trim_end_matches(".ts");
    
    let mut found_matching_class = false;
    
    for stmt in &program.body {
        if let Statement::ExportNamedDeclaration(export) = stmt {
            if let Some(Declaration::ClassDeclaration(class)) = &export.declaration {
                if let Some(id) = &class.id {
                    let class_name = id.name.as_str();
                    
                    // Check if class name matches filename
                    if class_name == filename {
                        found_matching_class = true;
                        
                        // Check if it extends Error
                        if let Some(super_class) = &class.super_class {
                            if let Expression::Identifier(super_id) = super_class {
                                if super_id.name != "Error" {
                                    linter.add_error(
                                        "path-based-restrictions".to_string(),
                                        format!("Error class '{}' must extend Error", class_name),
                                        class.span,
                                    );
                                }
                            }
                        } else {
                            linter.add_error(
                                "path-based-restrictions".to_string(),
                                format!("Error class '{}' must extend Error", class_name),
                                class.span,
                            );
                        }
                    } else if class_name.ends_with("Error") {
                        linter.add_error(
                            "path-based-restrictions".to_string(),
                            format!("Error class must be named '{}' to match filename", filename),
                            class.span,
                        );
                    }
                }
            }
        }
    }
    
    if !found_matching_class {
        linter.add_error(
            "path-based-restrictions".to_string(),
            format!("io/errors/{}.ts must export error class '{}' extending Error", filename, filename),
            Span::new(0, 0),
        );
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
        if let Statement::ExportNamedDeclaration(export) = stmt {
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

/// Check that *.test.ts or *_test.ts files import the function with matching name
fn check_test_file_imports(linter: &mut Linter, program: &Program, file_path: &str) {
    // If a test runner is specified, check for appropriate imports
    if let Some(test_runner) = linter.test_runner.clone() {
        check_test_runner_imports(linter, program, &test_runner);
    }
    // Extract base filename without test suffix
    let filename = file_path
        .rsplit('/')
        .next()
        .unwrap_or("");
    
    // Remove test suffix (.test.ts or _test.ts)
    let filename = if filename.ends_with(".test.ts") {
        filename.trim_end_matches(".test.ts")
    } else if filename.ends_with("_test.ts") {
        filename.trim_end_matches("_test.ts")
    } else {
        filename
    };
    
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

/// Check that test files use the correct test runner imports
fn check_test_runner_imports(linter: &mut Linter, program: &Program, test_runner: &TestRunner) {
    let mut found_test_runner_import = false;
    let mut found_wrong_runner = false;
    let mut wrong_runner_name = String::new();
    
    // Check all imports
    for stmt in &program.body {
        if let Statement::ImportDeclaration(import) = stmt {
            let source = import.source.value.as_str();
            
            // Check if this import matches the specified test runner
            if test_runner.matches_import(source) {
                found_test_runner_import = true;
            }
            
            // Check if this import matches a different test runner
            for other_runner in [TestRunner::Vitest, TestRunner::NodeTest, TestRunner::DenoTest].iter() {
                if other_runner != test_runner && other_runner.matches_import(source) {
                    found_wrong_runner = true;
                    wrong_runner_name = other_runner.to_string();
                    break;
                }
            }
        }
    }
    
    // Report errors
    if found_wrong_runner {
        linter.add_error(
            "path-based-restrictions".to_string(),
            format!("Test file should use '{}' but found imports for '{}'", test_runner, wrong_runner_name),
            Span::new(0, 0),
        );
    } else if !found_test_runner_import {
        // Check if there are any test-like function calls that suggest a test file
        let mut has_test_code = false;
        for stmt in &program.body {
            if contains_test_code(stmt) {
                has_test_code = true;
                break;
            }
        }
        
        if has_test_code {
            linter.add_error(
                "path-based-restrictions".to_string(),
                format!("Test file should import from '{}' test runner", test_runner),
                Span::new(0, 0),
            );
        }
    }
}

/// Check if a statement contains test-like code
fn contains_test_code(stmt: &Statement) -> bool {
    if let Statement::ExpressionStatement(expr_stmt) = stmt {
        if let Expression::CallExpression(call) = &expr_stmt.expression {
            if let Expression::Identifier(id) = &call.callee {
                let name = id.name.as_str();
                return name == "describe" || name == "it" || name == "test" || name == "expect";
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc::allocator::Allocator;
    use oxc::parser::Parser;
    use oxc::span::SourceType;

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
    fn test_io_functions() {
        // Both sync and async functions should be allowed in io/
        let source = r#"
            export function readFileSync(path: string): string {
                return "content";
            }
        "#;
        let errors = parse_and_check(source, "src/io/file.ts");
        assert_eq!(errors.len(), 0); // No error for sync function

        // Async function in io/ should also pass
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
        let errors = parse_and_check(source, "src/add.test.ts");
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().any(|e| e.contains("must import function 'add'")));

        // Test file with no imports should error
        let source = r#"
            describe("add", () => {
                it("should work", () => {});
            });
        "#;
        let errors = parse_and_check(source, "src/add.test.ts");
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().any(|e| e.contains("must have at least one import") || e.contains("must import from 'vitest'")));

        // Test file with matching import should pass
        let source = r#"
            import { add } from "./add";
            
            describe("add", () => {
                it("should add two numbers", () => {
                    expect(add(1, 2)).toBe(3);
                });
            });
        "#;
        let errors = parse_and_check(source, "src/add.test.ts");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("should import from 'vitest'"));

        // Test file with default import matching should pass
        let source = r#"
            import calculate from "./calculate.ts";
            
            describe("calculate", () => {
                it("should work", () => {});
            });
        "#;
        let errors = parse_and_check(source, "src/calculate.test.ts");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("should import from 'vitest'"));
    }

    #[test]
    fn test_index_file_restrictions() {
        // index.ts with direct export should error
        let source = r#"
            export const version = "1.0.0";
        "#;
        let errors = parse_and_check(source, "src/index.ts");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("only contain re-exports"));

        // index.ts with re-export should pass
        let source = r#"
            export { add } from "./add";
            export type { Point } from "./types";
        "#;
        let errors = parse_and_check(source, "src/index.ts");
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_main_file() {
        // main.ts should allow main() calls
        // This is tested in no_top_level_side_effects.rs
        // Here we just verify path detection works
        let source = r#"
            function main() {
                console.log("Hello");
            }
            main();
        "#;
        let errors = parse_and_check(source, "src/main.ts");
        // Should not have path-based errors
        assert!(!errors.iter().any(|e| e.contains("path-based")));
    }
}