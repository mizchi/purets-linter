use oxc_ast::ast::*;
use oxc_ast::Visit;
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;
use std::collections::{HashMap, HashSet};

use crate::Linter;

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
}

impl<'a> CombinedVisitor<'a> {
    pub fn new(linter: &'a mut Linter) -> Self {
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
        }
    }
    
    pub fn check_program(&mut self, program: &'a Program<'a>) {
        // First pass: collect exports for one-public-function rule
        self.collect_exports(program);
        
        // Visit the entire program
        self.visit_program(program);
        
        // Post-processing checks
        self.check_one_public_function();
        self.check_unused_variables();
        self.check_prefer_readonly_arrays();
    }
    
    fn collect_exports(&mut self, program: &'a Program<'a>) {
        for item in &program.body {
            match item {
                Statement::ExportNamedDeclaration(export) => {
                    // Check for re-exports
                    if export.source.is_some() && !export.specifiers.is_empty() {
                        self.linter.add_error(
                            "no-reexports".to_string(),
                            format!("Re-exports from '{}' are not allowed", 
                                export.source.as_ref().unwrap().value),
                            export.span,
                        );
                    }
                    
                    // Check for named exports
                    if export.declaration.is_some() || !export.specifiers.is_empty() {
                        self.linter.add_error(
                            "no-named-exports".to_string(),
                            "Named exports are not allowed. Use default export only".to_string(),
                            export.span,
                        );
                    }
                    
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
                    self.linter.add_error(
                        "no-reexports".to_string(),
                        format!("Re-exports from '{}' are not allowed", export.source.value),
                        export.span,
                    );
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
    // Check for classes (no-classes rule)
    fn visit_class(&mut self, class: &Class<'a>) {
        self.linter.add_error(
            "no-classes".to_string(),
            "Classes are not allowed in pure TypeScript subset".to_string(),
            class.span,
        );
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
        self.linter.add_error(
            "no-throw".to_string(),
            "Throw statements are not allowed. Use Result type instead".to_string(),
            stmt.span,
        );
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
                }
            }
        }
        
        // Check for eval
        if let Expression::Identifier(ident) = &call.callee {
            if ident.name == "eval" {
                self.linter.add_error(
                    "no-eval-function".to_string(),
                    "eval() is not allowed".to_string(),
                    call.span,
                );
            }
        }
        
        oxc_ast::visit::walk::walk_call_expression(self, call);
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
    
    // Check for namespace imports, import extensions, and HTTP imports
    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        // Check namespace imports
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
        
        let source = &import.source.value;
        
        // Check HTTP(S) imports
        if source.starts_with("http://") || source.starts_with("https://") {
            self.linter.add_error(
                "no-http-imports".to_string(),
                format!("HTTP(S) imports are not allowed. Import from '{}' is forbidden", source),
                import.span,
            );
        }
        
        // Check import extensions
        if source.starts_with('.') || source.starts_with("../") {
            if !source.ends_with(".js") && !source.ends_with(".mjs") 
                && !source.ends_with(".cjs") && !source.ends_with(".json") {
                self.linter.add_error(
                    "import-extensions".to_string(),
                    format!("Relative imports must have an extension. Change '{}' to '{}.js'", source, source),
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
    
    // Track variable usage
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.used_vars.insert(ident.name.to_string());
        oxc_ast::visit::walk::walk_identifier_reference(self, ident);
    }
    
    // Check for top-level side effects and unused map
    fn visit_expression_statement(&mut self, stmt: &ExpressionStatement<'a>) {
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
                
                if !is_iife {
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
        if self.in_function {
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
        if let AssignmentTarget::StaticMemberExpression(_) = &expr.left {
            self.linter.add_error(
                "no-member-assignments".to_string(),
                "Direct member assignments are not allowed".to_string(),
                expr.span,
            );
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
    
    // Check for as casts (no-as-cast rule)
    fn visit_ts_as_expression(&mut self, expr: &TSAsExpression<'a>) {
        self.linter.add_error(
            "no-as-cast".to_string(),
            "Type assertions with 'as' are not allowed".to_string(),
            expr.span,
        );
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
    
    // Check for mutable Record (no-mutable-record rule)
    fn visit_ts_type_reference(&mut self, type_ref: &TSTypeReference<'a>) {
        if let TSTypeName::IdentifierReference(id) = &type_ref.type_name {
            if id.name == "Record" {
                self.linter.add_error(
                    "no-mutable-record".to_string(),
                    "Mutable Record<K, V> is not allowed. Use ReadonlyMap or define a specific interface".to_string(),
                    type_ref.span,
                );
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
    let mut visitor = CombinedVisitor::new(linter);
    visitor.check_program(program);
}