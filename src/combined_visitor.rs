use oxc_ast::ast::*;
use oxc_ast::Visit;
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;
use std::collections::{HashMap, HashSet};

use crate::{Linter, rules::{AllowedFeatures, UsedFeatures}};

/// Combined visitor that performs all rule checks in a single AST traversal
pub struct CombinedVisitor<'a> {
    linter: &'a mut Linter,
    // State for various rules
    exported_functions: Vec<(&'a str, oxc_span::Span)>,
    exported_other: Vec<(&'a str, oxc_span::Span)>,
    declared_vars: HashSet<String>,
    used_vars: HashSet<String>,
    in_catch_block: bool,
    current_catch_param: Option<String>,
    // State for no-this-in-functions
    in_function: bool,
    // State for prefer-readonly-array
    array_variables: HashMap<String, oxc_span::Span>,
    mutated_arrays: HashSet<String>,
    readonly_arrays: HashSet<String>,
    // State for no-global-process
    imported_process_names: HashSet<String>,
    // State for no-side-effect-functions
    in_default_parameter: bool,
    // State for @allow directives
    allowed_features: AllowedFeatures,
    used_features: UsedFeatures,
    // Special file types
    is_error_file: bool,
}

impl<'a> CombinedVisitor<'a> {
    pub fn new(linter: &'a mut Linter) -> Self {
        // Parse @allow directives from the source
        let allowed_features = AllowedFeatures::from_jsdoc(&linter.source_text);
        
        // Check if this is an error file
        let path_str = linter.path.to_str().unwrap_or("").replace('\\', "/");
        let is_error_file = path_str.contains("/errors/");
        
        Self {
            linter,
            exported_functions: Vec::new(),
            exported_other: Vec::new(),
            declared_vars: HashSet::new(),
            used_vars: HashSet::new(),
            in_catch_block: false,
            current_catch_param: None,
            in_function: false,
            array_variables: HashMap::new(),
            mutated_arrays: HashSet::new(),
            readonly_arrays: HashSet::new(),
            imported_process_names: HashSet::new(),
            in_default_parameter: false,
            allowed_features,
            used_features: UsedFeatures::default(),
            is_error_file,
        }
    }
    
    pub fn check_program(&mut self, program: &'a Program<'a>) {
        // First pass: collect exports and imports
        self.collect_exports(program);
        self.collect_imports(program);
        
        // Check path-based restrictions (io/pure/types conventions)
        let file_path = self.linter.path.to_str().unwrap_or("").to_string();
        crate::rules::check_path_based_restrictions(self.linter, program, &file_path);
        
        // Check filename-function match
        self.check_filename_function_match(program);
        
        // Check JSDoc for exports
        self.check_export_jsdoc(program);
        
        // Visit the entire program
        self.visit_program(program);
        
        // Post-processing checks
        self.check_one_public_function();
        self.check_unused_variables();
        self.check_prefer_readonly_arrays();
        self.check_unused_allow_directives();
    }
    
    fn check_unused_allow_directives(&mut self) {
        if self.allowed_features.dom && !self.used_features.dom {
            self.linter.add_error(
                "allow-directives".to_string(),
                "Unused '@allow dom' directive".to_string(),
                oxc_span::Span::new(0, 0),
            );
        }
        if self.allowed_features.net && !self.used_features.net {
            self.linter.add_error(
                "allow-directives".to_string(),
                "Unused '@allow net' directive".to_string(),
                oxc_span::Span::new(0, 0),
            );
        }
        if self.allowed_features.timers && !self.used_features.timers {
            self.linter.add_error(
                "allow-directives".to_string(),
                "Unused '@allow timers' directive".to_string(),
                oxc_span::Span::new(0, 0),
            );
        }
        if self.allowed_features.console && !self.used_features.console {
            self.linter.add_error(
                "allow-directives".to_string(),
                "Unused '@allow console' directive".to_string(),
                oxc_span::Span::new(0, 0),
            );
        }
        if self.allowed_features.throws && !self.used_features.throws {
            self.linter.add_error(
                "allow-directives".to_string(),
                "Unused '@allow throws' directive".to_string(),
                oxc_span::Span::new(0, 0),
            );
        }
    }
    
    fn collect_imports(&mut self, program: &'a Program<'a>) {
        for item in &program.body {
            if let Statement::ImportDeclaration(import) = item {
                // Check if importing process from node:process
                if import.source.value == "node:process" || import.source.value == "process" {
                    if let Some(specifiers) = &import.specifiers {
                        for spec in specifiers {
                            match spec {
                                ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                                    self.imported_process_names.insert(default.local.name.to_string());
                                }
                                ImportDeclarationSpecifier::ImportSpecifier(named) => {
                                    self.imported_process_names.insert(named.local.name.to_string());
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn collect_exports(&mut self, program: &'a Program<'a>) {
        for item in &program.body {
            match item {
                Statement::ExportNamedDeclaration(export) => {
                    // Check for re-exports (skip for entry points/index files)
                    let filename = self.linter.path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    let is_entry_point = filename == "index" || 
                                        self.linter.is_entry_point || 
                                        self.linter.is_main_entry;
                    
                    if !is_entry_point && export.source.is_some() && !export.specifiers.is_empty() {
                        self.linter.add_error(
                            "no-reexports".to_string(),
                            format!("Re-exports from '{}' are not allowed", 
                                export.source.as_ref().unwrap().value),
                            export.span,
                        );
                    }
                    
                    // Named export checking is now handled by strict_named_export rule
                    
                    if let Some(Declaration::FunctionDeclaration(func)) = &export.declaration {
                        if let Some(id) = &func.id {
                            self.exported_functions.push((id.name.as_str(), export.span));
                        }
                    } else if let Some(Declaration::VariableDeclaration(var_decl)) = &export.declaration {
                        // Check export const type required
                        if var_decl.kind == VariableDeclarationKind::Let {
                            self.linter.add_error(
                                "export-const-type-required".to_string(),
                                "Exported 'let' declarations are not allowed. Use 'const' instead".to_string(),
                                export.span,
                            );
                        }
                        
                        for decl in &var_decl.declarations {
                            if let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind {
                                // Check for type annotation on exported const
                                if var_decl.kind == VariableDeclarationKind::Const && decl.id.type_annotation.is_none() {
                                    self.linter.add_error(
                                        "export-const-type-required".to_string(),
                                        format!("Exported const '{}' requires type annotation", id.name),
                                        decl.span,
                                    );
                                }
                                
                                if let Some(init) = &decl.init {
                                    if matches!(init, Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)) {
                                        self.exported_functions.push((id.name.as_str(), export.span));
                                    } else {
                                        self.exported_other.push((id.name.as_str(), export.span));
                                    }
                                }
                            }
                        }
                    }
                }
                Statement::ExportAllDeclaration(export) => {
                    // Check for re-exports (skip for entry points/index files)
                    let filename = self.linter.path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    let is_entry_point = filename == "index" || 
                                        self.linter.is_entry_point || 
                                        self.linter.is_main_entry;
                    
                    if !is_entry_point {
                        self.linter.add_error(
                            "no-reexports".to_string(),
                            format!("Re-exports from '{}' are not allowed", export.source.value),
                            export.span,
                        );
                    } else {
                        // For entry points, namespace re-exports are still not allowed
                        self.linter.add_error(
                            "no-reexports".to_string(),
                            format!("Namespace re-exports are not allowed in entry points. Use named exports: export {{ name }} from '{}'", export.source.value),
                            export.span,
                        );
                    }
                }
                Statement::ExportDefaultDeclaration(export) => {
                    match &export.declaration {
                        ExportDefaultDeclarationKind::FunctionDeclaration(_) => {
                            self.exported_functions.push(("default", export.span));
                        }
                        _ => {
                            self.exported_other.push(("default", export.span));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    fn check_one_public_function(&mut self) {
        if !self.exported_other.is_empty() {
            for (name, span) in &self.exported_other {
                self.linter.add_error(
                    "one-public-function".to_string(),
                    format!("Only functions can be exported. Found non-function export: {}", name),
                    *span,
                );
            }
        }
        
        if self.exported_functions.len() > 1 {
            for (name, span) in &self.exported_functions[1..] {
                self.linter.add_error(
                    "one-public-function".to_string(),
                    format!("Only one function can be exported per file. Found additional export: {}", name),
                    *span,
                );
            }
        }
    }
    
    fn check_unused_variables(&mut self) {
        for var in &self.declared_vars {
            if !self.used_vars.contains(var) {
                // Note: In a real implementation, we'd need the span of the declaration
                // This is simplified for demonstration
                self.linter.add_error(
                    "no-unused-variables".to_string(),
                    format!("Variable '{}' is declared but never used", var),
                    oxc_span::Span::new(0, 0),
                );
            }
        }
    }
    
    fn check_prefer_readonly_arrays(&mut self) {
        for (name, span) in &self.array_variables {
            if !self.mutated_arrays.contains(name) && !self.readonly_arrays.contains(name) {
                self.linter.add_error(
                    "prefer-readonly-array".to_string(),
                    format!(
                        "Array '{}' is never mutated. Consider using 'ReadonlyArray' or 'readonly' modifier",
                        name
                    ),
                    *span,
                );
            }
        }
    }
    
    fn check_filename_function_match(&mut self, _program: &'a Program<'a>) {
        // Filename-function match is now handled by the individual rule
    }
    
    fn check_export_jsdoc(&mut self, program: &'a Program<'a>) {
        let source_text = self.linter.source_text.clone();
        
        for item in &program.body {
            match item {
                Statement::ExportDefaultDeclaration(export) => {
                    if let ExportDefaultDeclarationKind::FunctionDeclaration(func) = &export.declaration {
                        if !self.has_jsdoc_before(export.span, &source_text) {
                            let name = func.id.as_ref()
                                .map(|id| id.name.as_str())
                                .unwrap_or("anonymous");
                            self.linter.add_error(
                                "export-requires-jsdoc".to_string(),
                                format!("Exported function '{}' must have a JSDoc comment", name),
                                export.span,
                            );
                        }
                    }
                }
                Statement::ExportNamedDeclaration(export) => {
                    if let Some(Declaration::FunctionDeclaration(func)) = &export.declaration {
                        if !self.has_jsdoc_before(export.span, &source_text) {
                            let name = func.id.as_ref()
                                .map(|id| id.name.as_str())
                                .unwrap_or("anonymous");
                            self.linter.add_error(
                                "export-requires-jsdoc".to_string(),
                                format!("Exported function '{}' must have a JSDoc comment", name),
                                export.span,
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    fn has_jsdoc_before(&self, span: oxc_span::Span, source_text: &str) -> bool {
        let text_before = &source_text[..span.start as usize];
        let trimmed = text_before.trim_end();
        trimmed.ends_with("*/") && {
            if let Some(_comment_start) = trimmed.rfind("/**") {
                let between = &source_text[trimmed.len()..span.start as usize];
                between.trim().is_empty()
            } else {
                false
            }
        }
    }
    
    fn is_array_type(&self, type_ann: &TSTypeAnnotation) -> bool {
        match &type_ann.type_annotation {
            TSType::TSArrayType(_) => true,
            TSType::TSTypeReference(type_ref) => {
                if let TSTypeName::IdentifierReference(id) = &type_ref.type_name {
                    id.name == "Array"
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    
    fn is_readonly_array_type(&self, type_ann: &TSTypeAnnotation) -> bool {
        match &type_ann.type_annotation {
            TSType::TSTypeReference(type_ref) => {
                if let TSTypeName::IdentifierReference(id) = &type_ref.type_name {
                    id.name == "ReadonlyArray"
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

impl<'a> Visit<'a> for CombinedVisitor<'a> {
    // Track function and parameter state for side-effect checks
    fn visit_formal_parameter(&mut self, param: &FormalParameter<'a>) {
        // Check if we're in a default parameter
        if param.pattern.optional {
            self.in_default_parameter = true;
        }
        
        oxc_ast::visit::walk::walk_formal_parameter(self, param);
        
        self.in_default_parameter = false;
    }
    // Check for classes (no-classes rule)
    // Classes are now checked by the individual no_classes rule which handles extends Error
    fn visit_class(&mut self, class: &Class<'a>) {
        oxc_ast::visit::walk::walk_class(self, class);
    }
    
    // Check for enums (no-enums rule)
    fn visit_ts_enum_declaration(&mut self, decl: &TSEnumDeclaration<'a>) {
        self.linter.add_error(
            "no-enums".to_string(),
            "Enums are not allowed in pure TypeScript subset".to_string(),
            decl.span,
        );
        oxc_ast::visit::walk::walk_ts_enum_declaration(self, decl);
    }
    
    // Check for delete operator (no-delete rule)
    fn visit_unary_expression(&mut self, expr: &UnaryExpression<'a>) {
        if expr.operator == UnaryOperator::Delete {
            self.linter.add_error(
                "no-delete".to_string(),
                "Delete operator is not allowed".to_string(),
                expr.span,
            );
        }
        oxc_ast::visit::walk::walk_unary_expression(self, expr);
    }
    
    // Check for throw statements (no-throw rule)
    fn visit_throw_statement(&mut self, stmt: &ThrowStatement<'a>) {
        // Skip if @allow throws is specified
        if !self.allowed_features.throws {
            self.linter.add_error(
                "no-throw".to_string(),
                "Throw statements are not allowed. Use Result type instead".to_string(),
                stmt.span,
            );
        } else {
            self.used_features.throws = true;
        }
        oxc_ast::visit::walk::walk_throw_statement(self, stmt);
    }
    
    // Check for forEach, eval, Object.defineProperty, and track array mutations
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        // Check for forEach, Object.defineProperty, and track array mutations
        if let Some(member) = call.callee.as_member_expression() {
            if let MemberExpression::StaticMemberExpression(static_member) = &member {
                let method_name = static_member.property.name.as_str();
                
                if method_name == "forEach" {
                    self.linter.add_error(
                        "no-foreach".to_string(),
                        "forEach is not allowed. Use for...of loop instead".to_string(),
                        call.span,
                    );
                }
                
                // Check for Object.defineProperty and Object.defineProperties
                if let Expression::Identifier(obj) = &static_member.object {
                    if obj.name == "Object" {
                        if method_name == "defineProperty" {
                            self.linter.add_error(
                                "no-define-property".to_string(),
                                "Object.defineProperty is not allowed. Use direct property assignment or object literals instead".to_string(),
                                call.span,
                            );
                        } else if method_name == "defineProperties" {
                            self.linter.add_error(
                                "no-define-property".to_string(),
                                "Object.defineProperties is not allowed. Use direct property assignment or object literals instead".to_string(),
                                call.span,
                            );
                        }
                    }
                    
                    // Track mutating array methods
                    let obj_name = obj.name.to_string();
                    if self.array_variables.contains_key(&obj_name) {
                        const MUTATING_METHODS: &[&str] = &[
                            "push", "pop", "shift", "unshift", "splice", 
                            "sort", "reverse", "fill", "copyWithin"
                        ];
                        if MUTATING_METHODS.contains(&method_name) {
                            self.mutated_arrays.insert(obj_name);
                        }
                    }
                    
                    // Check for side-effect functions (Math.random, Date.now) - always disallow
                    if self.in_function && !self.in_default_parameter {
                        if (obj.name == "Math" && method_name == "random") {
                            self.linter.add_error(
                                "no-side-effect-functions".to_string(),
                                "Direct use of 'Math.random()' is not allowed in functions. Pass it as a parameter or use a default parameter instead".to_string(),
                                call.span,
                            );
                        } else if (obj.name == "Date" && method_name == "now") {
                            self.linter.add_error(
                                "no-side-effect-functions".to_string(),
                                "Direct use of 'Date.now()' is not allowed in functions. Pass it as a parameter or use a default parameter instead".to_string(),
                                call.span,
                            );
                        }
                    }
                    
                    // Check console access
                    if obj.name == "console" {
                        if !self.allowed_features.console {
                            self.linter.add_error(
                                "allow-directives".to_string(),
                                "Use of 'console' requires '@allow console' directive".to_string(),
                                call.span,
                            );
                        } else {
                            self.used_features.console = true;
                        }
                    }
                }
            }
        }
        
        // Check for eval and require
        if let Expression::Identifier(ident) = &call.callee {
            if ident.name == "eval" {
                self.linter.add_error(
                    "no-eval-function".to_string(),
                    "eval() is not allowed".to_string(),
                    call.span,
                );
            } else if ident.name == "require" {
                self.linter.add_error(
                    "no-require".to_string(),
                    "require() is not allowed. Use ES6 import statements instead".to_string(),
                    call.span,
                );
                
                // Also check forbidden libraries in require
                if call.arguments.len() > 0 {
                    if let Argument::StringLiteral(lit) = &call.arguments[0] {
                        let source = lit.value.as_str();
                        
                        const FORBIDDEN_LIBRARIES: &[&str] = &[
                            "jquery", "lodash", "lodash/fp", "underscore", "rxjs",
                        ];
                        
                        const PREFER_ALTERNATIVES: &[(&str, &str)] = &[
                            ("minimist", "node:util parseArgs"),
                            ("yargs", "node:util parseArgs"),
                        ];
                        
                        if FORBIDDEN_LIBRARIES.contains(&source) || source.starts_with("lodash/") {
                            self.linter.add_error(
                                "forbidden-libraries".to_string(),
                                format!("Library '{}' is forbidden. Consider using modern alternatives", source),
                                call.span,
                            );
                        }
                        
                        for (lib, alternative) in PREFER_ALTERNATIVES {
                            if source == *lib {
                                self.linter.add_error(
                                    "forbidden-libraries".to_string(),
                                    format!("Library '{}' has a better alternative. Use '{}' instead", lib, alternative),
                                    call.span,
                                );
                            }
                        }
                    }
                }
            }
            
            // Check timer functions
            const TIMER_FUNCTIONS: &[&str] = &[
                "setTimeout", "setInterval", "setImmediate",
                "requestAnimationFrame", "requestIdleCallback",
            ];
            
            if TIMER_FUNCTIONS.contains(&ident.name.as_str()) {
                if !self.allowed_features.timers {
                    self.linter.add_error(
                        "allow-directives".to_string(),
                        format!("Use of '{}' requires '@allow timers' directive", ident.name),
                        call.span,
                    );
                } else {
                    self.used_features.timers = true;
                }
            }
            
            // Check fetch access
            if ident.name == "fetch" {
                if !self.allowed_features.net {
                    self.linter.add_error(
                        "allow-directives".to_string(),
                        "Use of 'fetch' requires '@allow net' directive".to_string(),
                        call.span,
                    );
                } else {
                    self.used_features.net = true;
                }
            }
        }
        
        oxc_ast::visit::walk::walk_call_expression(self, call);
    }
    
    // Check for new Date() side effect
    fn visit_new_expression(&mut self, new_expr: &NewExpression<'a>) {
        if self.in_function && !self.in_default_parameter {
            if let Expression::Identifier(ident) = &new_expr.callee {
                if ident.name == "Date" {
                    self.linter.add_error(
                        "no-side-effect-functions".to_string(),
                        "Direct use of 'new Date()' is not allowed in functions. Pass it as a parameter or use a default parameter instead".to_string(),
                        new_expr.span,
                    );
                }
            }
        }
        
        // Check WebSocket, XMLHttpRequest
        if let Expression::Identifier(ident) = &new_expr.callee {
            if ident.name == "WebSocket" || ident.name == "XMLHttpRequest" || ident.name == "EventSource" {
                if !self.allowed_features.net {
                    self.linter.add_error(
                        "allow-directives".to_string(),
                        format!("Use of '{}' requires '@allow net' directive", ident.name),
                        new_expr.span,
                    );
                } else {
                    self.used_features.net = true;
                }
            }
        }
        
        oxc_ast::visit::walk::walk_new_expression(self, new_expr);
    }
    
    // Check for do-while loops (no-do-while rule)
    fn visit_do_while_statement(&mut self, stmt: &DoWhileStatement<'a>) {
        self.linter.add_error(
            "no-do-while".to_string(),
            "do-while statements are not allowed. Use while instead".to_string(),
            stmt.span,
        );
        oxc_ast::visit::walk::walk_do_while_statement(self, stmt);
    }
    
    // Check for getters/setters (no-getters-setters rule)
    fn visit_method_definition(&mut self, method: &MethodDefinition<'a>) {
        match method.kind {
            MethodDefinitionKind::Get => {
                self.linter.add_error(
                    "no-getters-setters".to_string(),
                    format!("Getter '{}' is not allowed. Use regular methods instead", 
                        method.key.name().unwrap_or(std::borrow::Cow::Borrowed("unknown"))),
                    method.span,
                );
            }
            MethodDefinitionKind::Set => {
                self.linter.add_error(
                    "no-getters-setters".to_string(),
                    format!("Setter '{}' is not allowed. Use regular methods instead",
                        method.key.name().unwrap_or(std::borrow::Cow::Borrowed("unknown"))),
                    method.span,
                );
            }
            _ => {}
        }
        oxc_ast::visit::walk::walk_method_definition(self, method);
    }
    
    // Check for interfaces without extends (interface-extends-only rule)
    fn visit_ts_interface_declaration(&mut self, decl: &TSInterfaceDeclaration<'a>) {
        if decl.extends.is_none() || decl.extends.as_ref().map_or(true, |e| e.is_empty()) {
            self.linter.add_error(
                "interface-extends-only".to_string(),
                format!(
                    "Interface '{}' without extends is not allowed. Use 'type' instead",
                    decl.id.name.as_str()
                ),
                decl.span,
            );
        }
        oxc_ast::visit::walk::walk_ts_interface_declaration(self, decl);
    }
    
    // Check for namespace imports, import extensions, HTTP imports, Node.js import style, and forbidden libraries
    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        let source = &import.source.value;
        
        // Forbidden libraries
        const FORBIDDEN_LIBRARIES: &[&str] = &[
            "jquery", "lodash", "lodash/fp", "underscore", "rxjs",
        ];
        
        // Libraries with better alternatives
        const PREFER_ALTERNATIVES: &[(&str, &str)] = &[
            ("minimist", "node:util parseArgs"),
            ("yargs", "node:util parseArgs"),
        ];
        
        // Check for forbidden libraries
        if FORBIDDEN_LIBRARIES.contains(&source.as_str()) || source.starts_with("lodash/") {
            self.linter.add_error(
                "forbidden-libraries".to_string(),
                format!("Library '{}' is forbidden. Consider using modern alternatives", source),
                import.span,
            );
        }
        
        // Check for libraries with better alternatives
        for (lib, alternative) in PREFER_ALTERNATIVES {
            if source == *lib {
                self.linter.add_error(
                    "forbidden-libraries".to_string(),
                    format!("Library '{}' has a better alternative. Use '{}' instead", lib, alternative),
                    import.span,
                );
            }
        }
        
        // Node.js built-in modules list
        const NODE_BUILTINS: &[&str] = &[
            "assert", "async_hooks", "buffer", "child_process", "cluster", "console",
            "constants", "crypto", "dgram", "diagnostics_channel", "dns", "domain",
            "events", "fs", "http", "http2", "https", "inspector", "module", "net",
            "os", "path", "perf_hooks", "process", "punycode", "querystring",
            "readline", "repl", "stream", "string_decoder", "sys", "timers", "tls",
            "trace_events", "tty", "url", "util", "v8", "vm", "wasi", "worker_threads", "zlib"
        ];
        
        // Check if it's a Node.js built-in without node: prefix
        if NODE_BUILTINS.contains(&source.as_str()) {
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
        const PREFER_PROMISES: &[(&str, &str)] = &[
            ("fs", "fs/promises"),
            ("dns", "dns/promises"),
            ("stream", "stream/promises"),
            ("timers", "timers/promises"),
            ("readline", "readline/promises"),
        ];
        
        for (old, new) in PREFER_PROMISES {
            if source == *old || source == format!("node:{}", old).as_str() {
                self.linter.add_error(
                    "node-import-style".to_string(),
                    format!("Prefer promise-based API. Use 'node:{}' instead of '{}'", new, source),
                    import.span,
                );
                break;
            }
        }
        
        // Check namespace imports from node: modules
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
        } else {
            // Check general namespace imports (not from node:)
            if let Some(specifiers) = &import.specifiers {
                for spec in specifiers {
                    if matches!(spec, ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)) {
                        self.linter.add_error(
                            "no-namespace-imports".to_string(),
                            "Namespace imports are not allowed. Use named imports instead".to_string(),
                            import.span,
                        );
                        break;
                    }
                }
            }
        }
        
        // Check HTTP(S) imports
        if source.starts_with("http://") || source.starts_with("https://") {
            self.linter.add_error(
                "no-http-imports".to_string(),
                format!("HTTP(S) imports are not allowed. Import from '{}' is forbidden", source),
                import.span,
            );
        }
        
        // Check import extensions - require .ts extension for TypeScript files
        if source.starts_with('.') || source.starts_with("../") {
            if !source.ends_with(".ts") && !source.ends_with(".tsx") 
                && !source.ends_with(".js") && !source.ends_with(".jsx") 
                && !source.ends_with(".json") {
                self.linter.add_error(
                    "import-extensions".to_string(),
                    format!("Relative imports must have an extension. Change '{}' to '{}.ts'", source, source),
                    import.span,
                );
            }
        }
        
        oxc_ast::visit::walk::walk_import_declaration(self, import);
    }
    
    // Check for empty arrays without type (empty-array-requires-type rule) and track arrays
    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        // Track variable declarations for unused variables check
        if let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind {
            let var_name = id.name.to_string();
            self.declared_vars.insert(var_name.clone());
            
            // Track array variables for prefer-readonly-array
            if let Some(type_ann) = &decl.id.type_annotation {
                if self.is_array_type(type_ann) {
                    self.array_variables.insert(var_name.clone(), decl.span);
                } else if self.is_readonly_array_type(type_ann) {
                    self.readonly_arrays.insert(var_name.clone());
                }
            }
            
            // Check for empty array without type annotation
            if let Some(Expression::ArrayExpression(array)) = &decl.init {
                if array.elements.is_empty() && decl.id.type_annotation.is_none() {
                    self.linter.add_error(
                        "empty-array-requires-type".to_string(),
                        format!("Empty array '{}' requires type annotation", id.name),
                        decl.span,
                    );
                }
                // Track arrays initialized with array literals
                if decl.id.type_annotation.is_none() {
                    self.array_variables.insert(var_name.clone(), decl.span);
                }
            }
            
            // Track Array.from, Array.of, new Array()
            if let Some(init) = &decl.init {
                match init {
                    Expression::NewExpression(new_expr) => {
                        if let Expression::Identifier(ctor_id) = &new_expr.callee {
                            if ctor_id.name == "Array" {
                                self.array_variables.insert(var_name.clone(), decl.span);
                            }
                        }
                    }
                    Expression::CallExpression(call) => {
                        if let Some(member) = call.callee.as_member_expression() {
                            if let MemberExpression::StaticMemberExpression(static_member) = member {
                                if let Expression::Identifier(obj) = &static_member.object {
                                    if obj.name == "Array" {
                                        self.array_variables.insert(var_name.clone(), decl.span);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        oxc_ast::visit::walk::walk_variable_declarator(self, decl);
    }
    
    // Track variable usage and check for global process/DOM access
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        let name = ident.name.to_string();
        self.used_vars.insert(name.clone());
        
        // Check for global process usage (no-global-process rule)
        if ident.name == "process" && !self.imported_process_names.contains(&name) {
            self.linter.add_error(
                "no-global-process".to_string(),
                "Global 'process' is not allowed. Import it from 'node:process' instead".to_string(),
                ident.span,
            );
        }
        
        // Check DOM globals
        const DOM_GLOBALS: &[&str] = &[
            "document", "window", "navigator", "location", 
            "localStorage", "sessionStorage", "history",
            "screen", "alert", "confirm", "prompt"
        ];
        
        if DOM_GLOBALS.contains(&ident.name.as_str()) {
            if !self.allowed_features.dom {
                self.linter.add_error(
                    "allow-directives".to_string(),
                    format!("Access to '{}' requires '@allow dom' directive", ident.name),
                    ident.span,
                );
            } else {
                self.used_features.dom = true;
            }
        }
        
        // Check network globals
        if ident.name == "XMLHttpRequest" || ident.name == "WebSocket" || 
           ident.name == "EventSource" || ident.name == "ServiceWorker" {
            if !self.allowed_features.net {
                self.linter.add_error(
                    "allow-directives".to_string(),
                    format!("Access to '{}' requires '@allow net' directive", ident.name),
                    ident.span,
                );
            } else {
                self.used_features.net = true;
            }
        }
        
        oxc_ast::visit::walk::walk_identifier_reference(self, ident);
    }
    
    // Check for top-level side effects and unused map
    fn visit_expression_statement(&mut self, stmt: &ExpressionStatement<'a>) {
        // Skip these checks for error files
        if self.is_error_file {
            oxc_ast::visit::walk::walk_expression_statement(self, stmt);
            return;
        }
        
        // Only check at top level (simplified check)
        match &stmt.expression {
            Expression::CallExpression(call) => {
                // Check if it's an IIFE
                let is_iife = match &call.callee {
                    Expression::FunctionExpression(_) | 
                    Expression::ArrowFunctionExpression(_) => true,
                    Expression::ParenthesizedExpression(paren) => {
                        matches!(&paren.expression, 
                            Expression::FunctionExpression(_) | 
                            Expression::ArrowFunctionExpression(_)
                        )
                    },
                    _ => false
                };
                
                // Skip top-level side effects check for test files and main/entry files
                let path_str = self.linter.path.to_str().unwrap_or("").replace('\\', "/");
                let filename = self.linter.path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                let is_test_file = path_str.contains("_test.ts") || path_str.contains(".test.ts");
                let is_main_or_entry = filename == "main" || filename == "index" || 
                                       self.linter.is_entry_point || self.linter.is_main_entry;
                
                if !is_iife && !is_test_file && !is_main_or_entry {
                    self.linter.add_error(
                        "no-top-level-side-effects".to_string(),
                        "Top-level function calls are not allowed (side effects)".to_string(),
                        stmt.span,
                    );
                }
                
                // Check for unused map
                if let Some(member) = call.callee.as_member_expression() {
                    if let MemberExpression::StaticMemberExpression(static_member) = &member {
                        if static_member.property.name == "map" {
                            self.linter.add_error(
                                "no-unused-map".to_string(),
                                "map() return value must be used".to_string(),
                                stmt.span,
                            );
                        }
                    }
                }
            }
            Expression::AssignmentExpression(_) => {
                self.linter.add_error(
                    "no-top-level-side-effects".to_string(),
                    "Top-level assignments are not allowed (side effects)".to_string(),
                    stmt.span,
                );
            }
            _ => {}
        }
        oxc_ast::visit::walk::walk_expression_statement(self, stmt);
    }
    
    // Check for this in functions and max params
    fn visit_function(&mut self, func: &Function<'a>, _flags: ScopeFlags) {
        // Check max function params (max-function-params rule)
        const MAX_PARAMS: usize = 2;
        let param_count = func.params.items.len();
        if param_count > MAX_PARAMS {
            let func_name = func.id.as_ref()
                .map(|id| id.name.as_str())
                .unwrap_or("<anonymous>");
            
            self.linter.add_error(
                "max-function-params".to_string(),
                format!(
                    "Function '{}' has {} parameters (max: {}). Use an options object as the second parameter instead",
                    func_name, param_count, MAX_PARAMS
                ),
                func.span,
            );
        }
        
        // Track function context for no-this-in-functions
        let was_in_function = self.in_function;
        self.in_function = true;
        oxc_ast::visit::walk::walk_function(self, func, _flags);
        self.in_function = was_in_function;
    }
    
    fn visit_this_expression(&mut self, expr: &ThisExpression) {
        // Skip this check for error files (allows this.name in constructor)
        if !self.is_error_file && self.in_function {
            self.linter.add_error(
                "no-this-in-functions".to_string(),
                "'this' is not allowed in regular functions".to_string(),
                expr.span,
            );
        }
        oxc_ast::visit::walk::walk_this_expression(self, expr);
    }
    
    // Check for filename/dirname (no-filename-dirname rule)
    fn visit_meta_property(&mut self, meta: &MetaProperty<'a>) {
        if meta.meta.name == "__filename" || meta.meta.name == "__dirname" {
            self.linter.add_error(
                "no-filename-dirname".to_string(),
                format!("'{}' is not allowed. Use import.meta.url instead", meta.meta.name),
                meta.span,
            );
        }
        oxc_ast::visit::walk::walk_meta_property(self, meta);
    }
    
    // Check for Object.assign, dynamic access, and member assignments
    fn visit_member_expression(&mut self, expr: &MemberExpression<'a>) {
        // Check for Object.assign
        if let MemberExpression::StaticMemberExpression(static_member) = expr {
            if let Expression::Identifier(obj) = &static_member.object {
                if obj.name == "Object" && static_member.property.name == "assign" {
                    self.linter.add_error(
                        "no-object-assign".to_string(),
                        "Object.assign is not allowed. Use spread operator instead".to_string(),
                        expr.span(),
                    );
                }
            }
        }
        
        // Check for dynamic property access (no-dynamic-access rule)
        if let MemberExpression::ComputedMemberExpression(computed) = expr {
            // Allow numeric indices for arrays
            let is_numeric = match &computed.expression {
                Expression::NumericLiteral(_) => true,
                Expression::StringLiteral(lit) => lit.value.parse::<i32>().is_ok(),
                _ => false,
            };
            
            if !is_numeric {
                self.linter.add_error(
                    "no-dynamic-access".to_string(),
                    "Dynamic property access is not allowed. Use dot notation or destructuring instead".to_string(),
                    computed.span,
                );
            }
        }
        
        oxc_ast::visit::walk::walk_member_expression(self, expr);
    }
    
    // Check for member assignments, dynamic assignments, and track array mutations
    fn visit_assignment_expression(&mut self, expr: &AssignmentExpression<'a>) {
        // Skip member assignment check for error files (allows this.name = "...")
        if !self.is_error_file {
            if let AssignmentTarget::StaticMemberExpression(_) = &expr.left {
                self.linter.add_error(
                    "no-member-assignments".to_string(),
                    "Direct member assignments are not allowed".to_string(),
                    expr.span,
                );
            }
        }
        
        // Check for dynamic property assignment and track array mutations
        if let AssignmentTarget::ComputedMemberExpression(member) = &expr.left {
            // Check if it's numeric (for arrays)
            let is_numeric = match &member.expression {
                Expression::NumericLiteral(_) => true,
                Expression::StringLiteral(lit) => lit.value.parse::<i32>().is_ok(),
                _ => false,
            };
            
            if !is_numeric {
                self.linter.add_error(
                    "no-dynamic-access".to_string(),
                    "Dynamic property assignment is not allowed. Use dot notation instead".to_string(),
                    member.span,
                );
            }
            
            // Track array element assignments
            if let Expression::Identifier(id) = &member.object {
                let name = id.name.to_string();
                if self.array_variables.contains_key(&name) {
                    self.mutated_arrays.insert(name);
                }
            }
        }
        
        oxc_ast::visit::walk::walk_assignment_expression(self, expr);
    }
    
    // Check for constant conditions (no-constant-condition rule)
    fn visit_if_statement(&mut self, stmt: &IfStatement<'a>) {
        if let Expression::BooleanLiteral(_) = &stmt.test {
            self.linter.add_error(
                "no-constant-condition".to_string(),
                "Avoid constant conditions in if statements".to_string(),
                stmt.test.span(),
            );
        }
        oxc_ast::visit::walk::walk_if_statement(self, stmt);
    }
    
    fn visit_while_statement(&mut self, stmt: &WhileStatement<'a>) {
        if let Expression::BooleanLiteral(lit) = &stmt.test {
            if lit.value {
                self.linter.add_error(
                    "no-constant-condition".to_string(),
                    "Avoid constant conditions in while statements".to_string(),
                    stmt.test.span(),
                );
            }
        }
        oxc_ast::visit::walk::walk_while_statement(self, stmt);
    }
    
    // Check for switch case blocks (switch-case-block rule)
    fn visit_switch_case(&mut self, case: &SwitchCase<'a>) {
        if case.consequent.len() > 1 {
            let has_block = case.consequent.iter().any(|stmt| {
                matches!(stmt, Statement::BlockStatement(_))
            });
            
            if !has_block {
                self.linter.add_error(
                    "switch-case-block".to_string(),
                    "Switch cases with multiple statements should use block scope".to_string(),
                    case.span,
                );
            }
        }
        oxc_ast::visit::walk::walk_switch_case(self, case);
    }
    
    // Check for as casts (no-as-cast rule) - but allow 'as const'
    fn visit_ts_as_expression(&mut self, expr: &TSAsExpression<'a>) {
        // Check if it's 'as const'
        let is_const_assertion = match &expr.type_annotation {
            TSType::TSTypeReference(type_ref) => {
                if let TSTypeName::IdentifierReference(id) = &type_ref.type_name {
                    id.name == "const"
                } else {
                    false
                }
            }
            _ => false,
        };
        
        if !is_const_assertion {
            self.linter.add_error(
                "no-as-cast".to_string(),
                "Type assertions with 'as' are not allowed (except 'as const')".to_string(),
                expr.span,
            );
        }
        
        oxc_ast::visit::walk::walk_ts_as_expression(self, expr);
    }
    
    // Check for let without type (let-requires-type rule)
    fn visit_variable_declaration(&mut self, decl: &VariableDeclaration<'a>) {
        if decl.kind == VariableDeclarationKind::Let {
            for declarator in &decl.declarations {
                if let BindingPatternKind::BindingIdentifier(id) = &declarator.id.kind {
                    if declarator.id.type_annotation.is_none() && declarator.init.is_none() {
                        self.linter.add_error(
                            "let-requires-type".to_string(),
                            format!("'let' declaration '{}' requires type annotation", id.name),
                            declarator.span,
                        );
                    }
                }
            }
        }
        oxc_ast::visit::walk::walk_variable_declaration(self, decl);
    }
    
    // Check for catch error handling (catch-error-handling rule)
    fn visit_catch_clause(&mut self, clause: &CatchClause<'a>) {
        let was_in_catch = self.in_catch_block;
        self.in_catch_block = true;
        
        if let Some(param) = &clause.param {
            if let BindingPatternKind::BindingIdentifier(id) = &param.pattern.kind {
                self.current_catch_param = Some(id.name.to_string());
            }
        }
        
        oxc_ast::visit::walk::walk_catch_clause(self, clause);
        
        self.in_catch_block = was_in_catch;
        self.current_catch_param = None;
    }
    
    // Check for mutable Record and DOM/Net types
    fn visit_ts_type_reference(&mut self, type_ref: &TSTypeReference<'a>) {
        if let TSTypeName::IdentifierReference(id) = &type_ref.type_name {
            let name = id.name.as_str();
            
            // Check mutable Record
            if name == "Record" {
                self.linter.add_error(
                    "no-mutable-record".to_string(),
                    "Mutable Record<K, V> is not allowed. Use ReadonlyMap or define a specific interface".to_string(),
                    type_ref.span,
                );
            }
            
            // Check DOM types
            const DOM_TYPES: &[&str] = &[
                "HTMLElement", "HTMLDivElement", "HTMLInputElement",
                "Document", "Window", "Navigator", "Location",
                "Element", "Node", "Event", "MouseEvent", "KeyboardEvent",
                "DOMParser", "XMLSerializer", "Storage"
            ];
            
            if DOM_TYPES.contains(&name) {
                if !self.allowed_features.dom {
                    self.linter.add_error(
                        "allow-directives".to_string(),
                        format!("Type '{}' requires '@allow dom' directive", name),
                        type_ref.span,
                    );
                } else {
                    self.used_features.dom = true;
                }
            }
            
            // Check network types
            const NET_TYPES: &[&str] = &[
                "Response", "Request", "Headers", "RequestInit",
                "XMLHttpRequest", "WebSocket", "EventSource",
                "ServiceWorker", "ServiceWorkerRegistration"
            ];
            
            if NET_TYPES.contains(&name) {
                if !self.allowed_features.net {
                    self.linter.add_error(
                        "allow-directives".to_string(),
                        format!("Type '{}' requires '@allow net' directive", name),
                        type_ref.span,
                    );
                } else {
                    self.used_features.net = true;
                }
            }
        }
        oxc_ast::visit::walk::walk_ts_type_reference(self, type_ref);
    }
    
    // Check arrow functions for max params
    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        const MAX_PARAMS: usize = 2;
        let param_count = arrow.params.items.len();
        if param_count > MAX_PARAMS {
            self.linter.add_error(
                "max-function-params".to_string(),
                format!(
                    "Arrow function has {} parameters (max: {}). Use an options object as the second parameter instead",
                    param_count, MAX_PARAMS
                ),
                arrow.span,
            );
        }
        
        oxc_ast::visit::walk::walk_arrow_function_expression(self, arrow);
    }
}

/// Use the combined visitor for efficient linting
pub fn check_program_combined(linter: &mut Linter, program: &Program) {
    // Run combined visitor for most rules
    let mut visitor = CombinedVisitor::new(linter);
    visitor.check_program(program);
    
    // Run individual rules that need special handling
    use crate::rules::{
        check_strict_named_export,
        check_filename_function_match,
        check_no_top_level_side_effects,
        check_path_based_restrictions,
        check_no_classes,
    };
    
    // Check if it's a test file or error class file
    let path_str = linter.path.to_str().unwrap_or("").to_string();
    let is_test_file = path_str.contains("_test.ts") || 
                       path_str.contains(".test.ts") || 
                       path_str.contains(".spec.ts");
    let is_error_file = path_str.contains("/errors/");
    
    // Apply no-classes rule (must check for extends Error)
    check_no_classes(linter, program);
    
    // Apply strict_named_export rule (replaces no-named-exports)
    check_strict_named_export(linter, program);
    
    // Apply filename_function_match rule
    check_filename_function_match(linter, program);
    
    // Apply no-top-level-side-effects rule only for non-test and non-error files
    if !is_test_file && !is_error_file {
        check_no_top_level_side_effects(linter, program);
    }
    
    // Apply path-based restrictions
    check_path_based_restrictions(linter, program, &path_str);
}