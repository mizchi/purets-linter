use oxc::ast::ast::*;

use crate::Linter;

pub fn check_no_http_imports(linter: &mut Linter, program: &Program) {
    for item in &program.body {
        if let Statement::ImportDeclaration(import) = item {
            let source = &import.source.value;
            if source.starts_with("http://") || source.starts_with("https://") {
                linter.add_error(
                    "no-http-imports".to_string(),
                    format!("HTTP(S) imports are not allowed. Import from '{}' is forbidden", source),
                    import.span,
                );
            }
        }
    }
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
        check_no_http_imports(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.message).collect()
    }

    #[test]
    fn test_https_import() {
        let source = r#"
            import React from "https://esm.sh/react@18";
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("HTTP(S) imports are not allowed"));
        assert!(errors[0].contains("https://esm.sh/react@18"));
    }

    #[test]
    fn test_http_import() {
        let source = r#"
            import { something } from "http://example.com/module.js";
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("HTTP(S) imports are not allowed"));
        assert!(errors[0].contains("http://example.com/module.js"));
    }

    #[test]
    fn test_https_with_path() {
        let source = r#"
            import mod from "https://deno.land/std@0.140.0/path/mod.ts";
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("https://deno.land/std@0.140.0/path/mod.ts"));
    }

    #[test]
    fn test_normal_imports_allowed() {
        let source = r#"
            import fs from "fs";
            import { join } from "path";
            import local from "./local.js";
            import pkg from "some-package";
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_multiple_http_imports() {
        let source = r#"
            import React from "https://esm.sh/react";
            import Vue from "https://esm.sh/vue";
            import normal from "normal-package";
            import axios from "http://unpkg.com/axios";
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 3); // 3 HTTP(S) imports
    }
}