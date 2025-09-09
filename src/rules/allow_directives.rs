use oxc_ast::ast::*;
use std::collections::HashSet;

use crate::Linter;

#[derive(Debug, Clone, Default)]
pub struct AllowedFeatures {
    pub timers: bool,
    pub console: bool,
    pub net: bool,
    pub dom: bool,
    pub throws: bool,
}

#[derive(Debug, Clone, Default)]
pub struct UsedFeatures {
    pub timers: bool,
    pub console: bool,
    pub net: bool,
    pub dom: bool,
    pub throws: bool,
}

impl AllowedFeatures {
    pub fn from_jsdoc(source_text: &str) -> Self {
        let mut features = Self::default();
        
        // Find the first JSDoc comment
        if let Some(jsdoc_start) = source_text.find("/**") {
            if let Some(jsdoc_end) = source_text[jsdoc_start..].find("*/") {
                let jsdoc = &source_text[jsdoc_start..jsdoc_start + jsdoc_end + 2];
                
                // Parse @allow directives
                for line in jsdoc.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("* @allow ") || trimmed.starts_with("*@allow ") {
                        let allow_text = trimmed
                            .trim_start_matches("*")
                            .trim()
                            .trim_start_matches("@allow")
                            .trim();
                        
                        match allow_text {
                            "timers" => features.timers = true,
                            "console" => features.console = true,
                            "net" => features.net = true,
                            "dom" => features.dom = true,
                            "throws" => features.throws = true,
                            _ => {}
                        }
                    }
                }
            }
        }
        
        features
    }
}

pub fn check_allow_directives(linter: &mut Linter, program: &Program) -> UsedFeatures {
    use oxc_ast::Visit;
    
    let allowed = AllowedFeatures::from_jsdoc(&linter.source_text);
    
    struct AllowDirectiveVisitor<'a, 'b> {
        linter: &'a mut Linter,
        allowed: AllowedFeatures,
        used: UsedFeatures,
        in_function: bool,
        _phantom: std::marker::PhantomData<&'b ()>,
    }
    
    impl<'a, 'b> Visit<'b> for AllowDirectiveVisitor<'a, 'b> {
        fn visit_function(&mut self, func: &Function<'b>, _: oxc_syntax::scope::ScopeFlags) {
            let was_in_function = self.in_function;
            self.in_function = true;
            
            oxc_ast::visit::walk::walk_function(self, func, oxc_syntax::scope::ScopeFlags::empty());
            
            self.in_function = was_in_function;
        }
        
        fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'b>) {
            let was_in_function = self.in_function;
            self.in_function = true;
            
            oxc_ast::visit::walk::walk_arrow_function_expression(self, arrow);
            
            self.in_function = was_in_function;
        }
        
        fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'b>) {
            let name = ident.name.as_str();
            
            // Check DOM access
            const DOM_GLOBALS: &[&str] = &[
                "document", "window", "navigator", "location", 
                "localStorage", "sessionStorage", "history",
                "screen", "alert", "confirm", "prompt"
            ];
            
            if DOM_GLOBALS.contains(&name) {
                if !self.allowed.dom {
                    self.linter.add_error(
                        "allow-directives".to_string(),
                        format!("Access to '{}' requires '@allow dom' directive", name),
                        ident.span,
                    );
                } else {
                    self.used.dom = true;
                }
            }
            
            // Check network access
            const NET_GLOBALS: &[&str] = &[
                "fetch", "XMLHttpRequest", "WebSocket", "EventSource",
                "ServiceWorker"
            ];
            
            if NET_GLOBALS.contains(&name) {
                if !self.allowed.net {
                    self.linter.add_error(
                        "allow-directives".to_string(),
                        format!("Access to '{}' requires '@allow net' directive", name),
                        ident.span,
                    );
                } else {
                    self.used.net = true;
                }
            }
            
            oxc_ast::visit::walk::walk_identifier_reference(self, ident);
        }
        
        fn visit_call_expression(&mut self, call: &CallExpression<'b>) {
            // Check timer functions
            if let Expression::Identifier(ident) = &call.callee {
                const TIMER_FUNCTIONS: &[&str] = &[
                    "setTimeout", "setInterval", "setImmediate",
                    "requestAnimationFrame", "requestIdleCallback",
                    "clearTimeout", "clearInterval", "clearImmediate",
                    "cancelAnimationFrame", "cancelIdleCallback"
                ];
                
                if TIMER_FUNCTIONS.contains(&ident.name.as_str()) {
                    if !self.allowed.timers {
                        self.linter.add_error(
                            "allow-directives".to_string(),
                            format!("Use of '{}' requires '@allow timers' directive", ident.name),
                            call.span,
                        );
                    } else {
                        self.used.timers = true;
                    }
                }
            }
            
            // Check console access
            if let Some(member) = call.callee.as_member_expression() {
                if let MemberExpression::StaticMemberExpression(static_member) = &member {
                    if let Expression::Identifier(obj) = &static_member.object {
                        if obj.name == "console" {
                            if !self.allowed.console {
                                self.linter.add_error(
                                    "allow-directives".to_string(),
                                    "Use of 'console' requires '@allow console' directive".to_string(),
                                    call.span,
                                );
                            } else {
                                self.used.console = true;
                            }
                        }
                    }
                }
            }
            
            oxc_ast::visit::walk::walk_call_expression(self, call);
        }
        
        fn visit_throw_statement(&mut self, throw_stmt: &ThrowStatement<'b>) {
            // Check if throw is allowed
            if !self.allowed.throws {
                // Check if it's throwing an Error constructor
                if let Expression::NewExpression(new_expr) = &throw_stmt.argument {
                    if let Expression::Identifier(id) = &new_expr.callee {
                        let name = id.name.as_str();
                        if name.ends_with("Error") {
                            self.linter.add_error(
                                "allow-directives".to_string(),
                                format!("Throwing '{}' requires '@allow throws' directive", name),
                                throw_stmt.span,
                            );
                        }
                    }
                } else {
                    self.linter.add_error(
                        "allow-directives".to_string(),
                        "Throw statements require '@allow throws' directive".to_string(),
                        throw_stmt.span,
                    );
                }
            } else {
                self.used.throws = true;
                
                // Check that only Error types are thrown
                if let Expression::NewExpression(new_expr) = &throw_stmt.argument {
                    if let Expression::Identifier(id) = &new_expr.callee {
                        let name = id.name.as_str();
                        if !name.ends_with("Error") {
                            self.linter.add_error(
                                "allow-directives".to_string(),
                                format!("Only Error types can be thrown (got '{}')", name),
                                throw_stmt.span,
                            );
                        }
                    }
                } else if !matches!(&throw_stmt.argument, Expression::Identifier(_)) {
                    // Allow throwing identifiers (like: throw error;)
                    // But disallow throwing literals or other expressions
                    if !matches!(&throw_stmt.argument, Expression::Identifier(_)) {
                        self.linter.add_error(
                            "allow-directives".to_string(),
                            "Only Error instances can be thrown".to_string(),
                            throw_stmt.span,
                        );
                    }
                }
            }
            
            oxc_ast::visit::walk::walk_throw_statement(self, throw_stmt);
        }
        
        fn visit_ts_type_reference(&mut self, type_ref: &TSTypeReference<'b>) {
            if let TSTypeName::IdentifierReference(id) = &type_ref.type_name {
                let name = id.name.as_str();
                
                // Check DOM type access
                if !self.allowed.dom {
                    const DOM_TYPES: &[&str] = &[
                        "HTMLElement", "HTMLDivElement", "HTMLInputElement",
                        "Document", "Window", "Navigator", "Location",
                        "Element", "Node", "Event", "MouseEvent", "KeyboardEvent",
                        "DOMParser", "XMLSerializer", "Storage"
                    ];
                    
                    if DOM_TYPES.contains(&name) {
                        self.linter.add_error(
                            "allow-directives".to_string(),
                            format!("Type '{}' requires '@allow dom' directive", name),
                            type_ref.span,
                        );
                    }
                }
                
                // Check network type access
                if !self.allowed.net {
                    const NET_TYPES: &[&str] = &[
                        "Response", "Request", "Headers", "RequestInit",
                        "XMLHttpRequest", "WebSocket", "EventSource",
                        "ServiceWorker", "ServiceWorkerRegistration"
                    ];
                    
                    if NET_TYPES.contains(&name) {
                        self.linter.add_error(
                            "allow-directives".to_string(),
                            format!("Type '{}' requires '@allow net' directive", name),
                            type_ref.span,
                        );
                    }
                }
            }
            
            oxc_ast::visit::walk::walk_ts_type_reference(self, type_ref);
        }
    }
    
    let mut visitor = AllowDirectiveVisitor {
        linter,
        allowed: allowed.clone(),
        used: UsedFeatures::default(),
        in_function: false,
        _phantom: std::marker::PhantomData,
    };
    
    visitor.visit_program(program);
    
    // Check for unused @allow directives
    if allowed.dom && !visitor.used.dom {
        visitor.linter.add_error(
            "allow-directives".to_string(),
            "Unused '@allow dom' directive".to_string(),
            oxc_span::Span::new(0, 0),
        );
    }
    if allowed.net && !visitor.used.net {
        visitor.linter.add_error(
            "allow-directives".to_string(),
            "Unused '@allow net' directive".to_string(),
            oxc_span::Span::new(0, 0),
        );
    }
    if allowed.timers && !visitor.used.timers {
        visitor.linter.add_error(
            "allow-directives".to_string(),
            "Unused '@allow timers' directive".to_string(),
            oxc_span::Span::new(0, 0),
        );
    }
    if allowed.console && !visitor.used.console {
        visitor.linter.add_error(
            "allow-directives".to_string(),
            "Unused '@allow console' directive".to_string(),
            oxc_span::Span::new(0, 0),
        );
    }
    
    visitor.used
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
        check_allow_directives(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.message).collect()
    }

    #[test]
    fn test_dom_without_allow() {
        let source = r#"
            function updateUI() {
                document.getElementById("app");
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("requires '@allow dom'"));
    }

    #[test]
    fn test_dom_with_allow() {
        let source = r#"
            /**
             * @allow dom
             */
            function updateUI() {
                document.getElementById("app");
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_timers_without_allow() {
        let source = r#"
            function delayed() {
                setTimeout(() => {}, 1000);
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("requires '@allow timers'"));
    }

    #[test]
    fn test_timers_with_allow() {
        let source = r#"
            /**
             * @allow timers
             */
            function delayed() {
                setTimeout(() => {}, 1000);
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_console_without_allow() {
        let source = r#"
            function debug() {
                console.log("debug");
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("requires '@allow console'"));
    }

    #[test]
    fn test_fetch_without_allow() {
        let source = r#"
            async function getData() {
                const res = await fetch("/api");
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("requires '@allow net'"));
    }

    #[test]
    fn test_multiple_allows() {
        let source = r#"
            /**
             * @allow dom
             * @allow net
             * @allow console
             */
            async function app() {
                console.log("starting");
                const data = await fetch("/api");
                document.body.innerHTML = data;
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_dom_types_without_allow() {
        let source = r#"
            function handleClick(event: MouseEvent): HTMLElement {
                return document.body;
            }
        "#;
        let errors = parse_and_check(source);
        assert!(errors.len() >= 2);
        assert!(errors.iter().any(|e| e.contains("'MouseEvent' requires '@allow dom'")));
        assert!(errors.iter().any(|e| e.contains("'HTMLElement' requires '@allow dom'")));
    }

    #[test]
    fn test_net_types_without_allow() {
        let source = r#"
            async function makeRequest(init: RequestInit): Promise<Response> {
                return fetch("/api", init);
            }
        "#;
        let errors = parse_and_check(source);
        assert!(errors.len() >= 2);
        assert!(errors.iter().any(|e| e.contains("'RequestInit' requires '@allow net'")));
        assert!(errors.iter().any(|e| e.contains("'Response' requires '@allow net'")));
    }

    #[test]
    fn test_unused_allow_directives() {
        let source = r#"
            /**
             * @allow dom
             * @allow console
             */
            function calculate(a: number, b: number): number {
                return a + b;
            }
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().any(|e| e.contains("Unused '@allow dom' directive")));
        assert!(errors.iter().any(|e| e.contains("Unused '@allow console' directive")));
    }

    #[test]
    fn test_used_allow_directives() {
        let source = r#"
            /**
             * @allow console
             */
            function debug() {
                console.log("test");
            }
        "#;
        let errors = parse_and_check(source);
        // Should not have unused directive error
        assert!(!errors.iter().any(|e| e.contains("Unused '@allow console'")));
    }

    #[test]
    fn test_partial_used_allow() {
        let source = r#"
            /**
             * @allow console
             * @allow dom
             * @allow net
             */
            function test() {
                console.log("test");
                // dom and net are not used
            }
        "#;
        let errors = parse_and_check(source);
        assert!(errors.iter().any(|e| e.contains("Unused '@allow dom' directive")));
        assert!(errors.iter().any(|e| e.contains("Unused '@allow net' directive")));
        assert!(!errors.iter().any(|e| e.contains("Unused '@allow console'")));
    }

    #[test]
    fn test_all_allow_features() {
        let source = r#"
            /**
             * @allow console
             * @allow dom
             * @allow net
             * @allow timers
             */
            async function testAll() {
                console.log("test");
                document.getElementById("app");
                await fetch("/api");
                setTimeout(() => {}, 1000);
            }
        "#;
        let errors = parse_and_check(source);
        // Should not have any unused directive errors
        assert!(!errors.iter().any(|e| e.contains("Unused '@allow")));
        // Should not have any access errors
        assert!(!errors.iter().any(|e| e.contains("requires '@allow")));
    }
}