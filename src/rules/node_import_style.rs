use oxc_ast::ast::*;

use crate::Linter;

// Common Node.js built-in modules
const NODE_BUILTINS: &[&str] = &[
    "assert", "async_hooks", "buffer", "child_process", "cluster", "console",
    "constants", "crypto", "dgram", "diagnostics_channel", "dns", "domain",
    "events", "fs", "http", "http2", "https", "inspector", "module", "net",
    "os", "path", "perf_hooks", "process", "punycode", "querystring",
    "readline", "repl", "stream", "string_decoder", "sys", "timers", "tls",
    "trace_events", "tty", "url", "util", "v8", "vm", "wasi", "worker_threads", "zlib"
];

// Modules that have promise-based versions we should prefer
const PREFER_PROMISES: &[(&str, &str)] = &[
    ("fs", "fs/promises"),
    ("dns", "dns/promises"),
    ("stream", "stream/promises"),
    ("timers", "timers/promises"),
    ("readline", "readline/promises"),
];

pub fn check_node_import_style(linter: &mut Linter, program: &Program) {
    use oxc_ast::Visit;
    
    struct NodeImportVisitor<'a, 'b> {
        linter: &'a mut Linter,
        _phantom: std::marker::PhantomData<&'b ()>,
    }
    
    impl<'a, 'b> Visit<'b> for NodeImportVisitor<'a, 'b> {
        fn visit_import_declaration(&mut self, import: &ImportDeclaration<'b>) {
            let source = import.source.value.as_str();
            
            // Check if it's a Node.js built-in without node: prefix
            if NODE_BUILTINS.contains(&source) {
                self.linter.add_error(
                    "node-import-style".to_string(),
                    format!(
                        "Node.js built-in '{}' must be imported with 'node:' prefix. Use 'node:{}' instead",
                        source, source
                    ),
                    import.span,
                );
            }
            
            // Check for modules that should use promises version
            for (old, new) in PREFER_PROMISES {
                if source == *old || source == format!("node:{}", old).as_str() {
                    self.linter.add_error(
                        "node-import-style".to_string(),
                        format!(
                            "Prefer promise-based API. Use 'node:{}' instead of '{}'",
                            new, source
                        ),
                        import.span,
                    );
                }
            }
            
            // Check for namespace imports from node: modules
            if source.starts_with("node:") {
                if let Some(specifiers) = &import.specifiers {
                    for spec in specifiers {
                        if matches!(spec, ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)) {
                            self.linter.add_error(
                                "node-import-style".to_string(),
                                format!(
                                    "Use named imports instead of namespace import from '{}'. Example: import {{ readFile }} from '{}'",
                                    source, source
                                ),
                                import.span,
                            );
                            break;
                        }
                    }
                }
            }
            
            oxc_ast::visit::walk::walk_import_declaration(self, import);
        }
    }
    
    let mut visitor = NodeImportVisitor {
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
        check_node_import_style(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.message).collect()
    }

    #[test]
    fn test_import_without_node_prefix() {
        let source = r#"
            import fs from 'fs';
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 2); // Missing node: prefix AND should use fs/promises
        assert!(errors[0].contains("must be imported with 'node:' prefix"));
    }

    #[test]
    fn test_import_fs_should_use_promises() {
        let source = r#"
            import { readFile } from 'node:fs';
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Use 'node:fs/promises' instead"));
    }

    #[test]
    fn test_namespace_import_from_node() {
        let source = r#"
            import * as fs from 'node:fs/promises';
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Use named imports instead of namespace import"));
    }

    #[test]
    fn test_correct_import() {
        let source = r#"
            import { readFile, writeFile } from 'node:fs/promises';
            import { join } from 'node:path';
            import process from 'node:process';
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_import_crypto_without_prefix() {
        let source = r#"
            import crypto from 'crypto';
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("'crypto' must be imported with 'node:' prefix"));
    }

    #[test]
    fn test_non_node_modules_allowed() {
        let source = r#"
            import React from 'react';
            import { useState } from 'react';
            import lodash from 'lodash';
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_dns_should_use_promises() {
        let source = r#"
            import { lookup } from 'node:dns';
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Use 'node:dns/promises' instead"));
    }
}