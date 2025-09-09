/// Common test utilities for rule testing
#[cfg(test)]
pub mod test {
    use crate::Linter;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;
    use std::path::Path;
    
    /// Parse and check with a specific rule
    pub fn check_rule<F>(source: &str, check_fn: F) -> Vec<String>
    where
        F: FnOnce(&mut Linter, &oxc_ast::ast::Program),
    {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new("test.ts")).unwrap();
        let ret = Parser::new(&allocator, source, source_type).parse();
        let mut linter = Linter::new(Path::new("test.ts"), source, false);
        check_fn(&mut linter, &ret.program);
        linter.errors.into_iter().map(|e| e.message).collect()
    }
    
    /// Parse and check with a specific file path
    pub fn check_rule_with_path<F>(source: &str, path: &str, check_fn: F) -> Vec<String>
    where
        F: FnOnce(&mut Linter, &oxc_ast::ast::Program),
    {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new(path)).unwrap();
        let ret = Parser::new(&allocator, source, source_type).parse();
        let mut linter = Linter::new(Path::new(path), source, false);
        check_fn(&mut linter, &ret.program);
        linter.errors.into_iter().map(|e| e.message).collect()
    }
    
    /// Assert that errors contain specific messages
    pub fn assert_errors_contain(errors: &[String], expected_messages: &[&str]) {
        for msg in expected_messages {
            assert!(
                errors.iter().any(|e| e.contains(msg)),
                "Expected error containing '{}', but got: {:?}",
                msg,
                errors
            );
        }
    }
    
    /// Assert no errors
    pub fn assert_no_errors(errors: &[String]) {
        assert!(
            errors.is_empty(),
            "Expected no errors, but got: {:?}",
            errors
        );
    }
}