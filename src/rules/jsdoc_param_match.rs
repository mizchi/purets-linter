use oxc_ast::ast::*;
use oxc_ast::visit::walk;
use oxc_ast::Visit;
use oxc_syntax::scope::ScopeFlags;

use crate::Linter;

pub fn check_jsdoc_param_match(linter: &mut Linter, program: &Program) {
    struct JsDocParamChecker<'a> {
        linter: &'a mut Linter,
        source_text: &'a str,
    }
    
    impl<'a> Visit<'a> for JsDocParamChecker<'a> {
        fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
            if func.id.is_some() {
                let func_name = func.id.as_ref().unwrap().name.as_str();
                self.check_jsdoc_params(func_name, &func.params.items, func.span);
            }
            walk::walk_function(self, func, flags);
        }
        
        fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
            // Arrow functions assigned to variables can have JSDoc
            // We'll check them when visiting variable declarations
            walk::walk_arrow_function_expression(self, arrow);
        }
        
        fn visit_variable_declaration(&mut self, decl: &VariableDeclaration<'a>) {
            for declarator in &decl.declarations {
                if let Some(init) = &declarator.init {
                    if let Expression::ArrowFunctionExpression(arrow) = init {
                        if let BindingPatternKind::BindingIdentifier(ident) = &declarator.id.kind {
                            let func_name = ident.name.as_str();
                            self.check_jsdoc_params(func_name, &arrow.params.items, arrow.span);
                        }
                    }
                }
            }
            walk::walk_variable_declaration(self, decl);
        }
    }
    
    impl<'a> JsDocParamChecker<'a> {
        fn check_jsdoc_params(&mut self, func_name: &str, params: &[FormalParameter<'a>], span: oxc_span::Span) {
            // Skip if no parameters
            if params.is_empty() {
                return;
            }
            
            // Extract JSDoc comment from source text
            let jsdoc_params = self.extract_jsdoc_params(span.start);
            
            // Check each function parameter
            for param in params {
                if let BindingPatternKind::BindingIdentifier(ident) = &param.pattern.kind {
                    let param_name = ident.name.as_str();
                    
                    // Check if parameter has TypeScript type annotation
                    if param.pattern.type_annotation.is_none() {
                        self.linter.add_error(
                            "param-missing-type".to_string(),
                            format!("Parameter '{}' in function '{}' must have a type ", param_name, func_name),
                            param.span,
                        );
                        continue;
                    }
                    
                    // Check if JSDoc exists for this parameter
                    if !jsdoc_params.is_empty()
                        && !jsdoc_params.iter().any(|(name, _)| name == param_name) {
                            self.linter.add_error(
                                "jsdoc-param-missing".to_string(),
                                format!("JSDoc @param tag missing for parameter '{}' in function '{}'", param_name, func_name),
                                param.span,
                            );
                        }
                }
            }
            
            // Check for JSDoc params that don't exist in function signature
            for (jsdoc_param_name, _) in &jsdoc_params {
                let exists = params.iter().any(|p| {
                    if let BindingPatternKind::BindingIdentifier(ident) = &p.pattern.kind {
                        ident.name.as_str() == jsdoc_param_name
                    } else {
                        false
                    }
                });
                
                if !exists {
                    self.linter.add_error(
                        "jsdoc-param-unknown".to_string(),
                        format!("JSDoc @param '{}' does not match any parameter in function '{}'", jsdoc_param_name, func_name),
                        span,
                    );
                }
            }
            
            // If function has JSDoc, require all params to be documented
            if !jsdoc_params.is_empty() && jsdoc_params.len() != params.len() {
                self.linter.add_error(
                    "jsdoc-param-count".to_string(),
                    format!("JSDoc has {} @param tags but function '{}' has {} parameters", jsdoc_params.len(), func_name, params.len()),
                    span,
                );
            }
        }
        
        fn extract_jsdoc_params(&self, func_start: u32) -> Vec<(String, String)> {
            let mut params = Vec::new();
            
            // Find the JSDoc comment before the function
            // Look for /** ... */ pattern
            let text_before = &self.source_text[..func_start as usize];
            
            if let Some(comment_end) = text_before.rfind("*/") {
                if let Some(comment_start) = text_before[..comment_end].rfind("/**") {
                    let comment = &text_before[comment_start + 3..comment_end];
                    
                    // Parse @param tags
                    for line in comment.lines() {
                        let trimmed = line.trim().trim_start_matches('*').trim();
                        if trimmed.starts_with("@param") {
                            // Parse: @param {type} name - description
                            let parts: Vec<&str> = trimmed["@param".len()..].trim().splitn(3, ' ').collect();
                            if parts.len() >= 2 {
                                // Extract type and name
                                let type_str = parts[0].trim_matches(|c| c == '{' || c == '}');
                                let name = parts[1].trim();
                                params.push((name.to_string(), type_str.to_string()));
                            } else if parts.len() == 1 {
                                // Just name, no type in JSDoc
                                let name = parts[0].trim();
                                params.push((name.to_string(), String::new()));
                            }
                        }
                    }
                }
            }
            
            params
        }
    }
    
    // Clone source_text to avoid borrow checker issues
    let source_text = linter.source_text.clone();
    let mut checker = JsDocParamChecker { 
        linter,
        source_text: &source_text,
    };
    checker.visit_program(program);
}
