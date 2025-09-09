use oxc_ast::ast::*;
use oxc_ast::visit::walk;
use oxc_ast::Visit;
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;

use crate::Linter;

pub fn check_no_classes(linter: &mut Linter, program: &Program) {
    struct ClassChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for ClassChecker<'a> {
        fn visit_class(&mut self, class: &Class<'a>) {
            self.linter.add_error(
                "no-classes".to_string(),
                "Classes are not allowed in pure TypeScript subset".to_string(),
                class.span,
            );
        }
    }
    
    let mut checker = ClassChecker { linter };
    checker.visit_program(program);
}

pub fn check_no_enums(linter: &mut Linter, program: &Program) {
    struct EnumChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for EnumChecker<'a> {
        fn visit_ts_enum_declaration(&mut self, decl: &TSEnumDeclaration<'a>) {
            self.linter.add_error(
                "no-enums".to_string(),
                "Enums are not allowed in pure TypeScript subset".to_string(),
                decl.span,
            );
        }
    }
    
    let mut checker = EnumChecker { linter };
    checker.visit_program(program);
}

pub fn check_no_reexports(linter: &mut Linter, program: &Program) {
    for item in &program.body {
        match item {
            Statement::ExportAllDeclaration(export) => {
                linter.add_error(
                    "no-reexports".to_string(),
                    "Re-exports are not allowed in pure TypeScript subset".to_string(),
                    export.span,
                );
            }
            Statement::ExportNamedDeclaration(export) if export.source.is_some() => {
                linter.add_error(
                    "no-reexports".to_string(),
                    "Re-exports are not allowed in pure TypeScript subset".to_string(),
                    export.span,
                );
            }
            _ => {}
        }
    }
}

pub fn check_no_namespace_imports(linter: &mut Linter, program: &Program) {
    struct NamespaceImportChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for NamespaceImportChecker<'a> {
        fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
            if let Some(specifiers) = &import.specifiers {
                for specifier in specifiers {
                    if matches!(specifier, ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)) {
                        self.linter.add_error(
                            "no-namespace-imports".to_string(),
                            "Namespace imports (import * as) are not allowed in pure TypeScript subset".to_string(),
                            import.span,
                        );
                    }
                }
            }
        }
    }
    
    let mut checker = NamespaceImportChecker { linter };
    checker.visit_program(program);
}

pub fn check_no_member_assignments(linter: &mut Linter, program: &Program) {
    struct MemberAssignmentChecker<'a> {
        linter: &'a mut Linter,
        in_function: bool,
    }
    
    impl<'a> Visit<'a> for MemberAssignmentChecker<'a> {
        fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
            let was_in_function = self.in_function;
            self.in_function = true;
            walk::walk_function(self, func, flags);
            self.in_function = was_in_function;
        }
        
        fn visit_assignment_expression(&mut self, expr: &AssignmentExpression<'a>) {
            if let AssignmentTarget::StaticMemberExpression(_member) = &expr.left {
                self.linter.add_error(
                    "no-member-assignments".to_string(),
                    format!("Member assignments like 'foo.bar = value' are not allowed in pure TypeScript subset"),
                    expr.span,
                );
            } else if let AssignmentTarget::ComputedMemberExpression(_) = &expr.left {
                self.linter.add_error(
                    "no-member-assignments".to_string(),
                    format!("Member assignments like 'foo[bar] = value' are not allowed in pure TypeScript subset"),
                    expr.span,
                );
            }
            walk::walk_assignment_expression(self, expr);
        }
    }
    
    let mut checker = MemberAssignmentChecker { 
        linter,
        in_function: false,
    };
    checker.visit_program(program);
}

pub fn check_one_public_function(linter: &mut Linter, program: &Program) {
    let mut exported_functions = Vec::new();
    let mut exported_other = Vec::new();
    
    for item in &program.body {
        match item {
            Statement::ExportNamedDeclaration(export) => {
                if let Some(Declaration::FunctionDeclaration(func)) = &export.declaration {
                    if let Some(id) = &func.id {
                        exported_functions.push((id.name.as_str(), export.span));
                    }
                } else if let Some(Declaration::VariableDeclaration(var_decl)) = &export.declaration {
                    for decl in &var_decl.declarations {
                        if let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind {
                            if let Some(init) = &decl.init {
                                if matches!(init, Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)) {
                                    exported_functions.push((id.name.as_str(), export.span));
                                } else {
                                    exported_other.push((id.name.as_str(), export.span));
                                }
                            }
                        }
                    }
                } else if export.declaration.is_some() {
                    exported_other.push(("(declaration)", export.span));
                }
                
                for spec in &export.specifiers {
                    exported_other.push((spec.exported.name().as_str(), export.span));
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                match &export.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(_) => {
                        exported_functions.push(("default", export.span));
                    }
                    _ if export.declaration.is_expression() => {
                        if let Some(expr) = export.declaration.as_expression() {
                            if matches!(expr, Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)) {
                                exported_functions.push(("default", export.span));
                            } else {
                                exported_other.push(("default", export.span));
                            }
                        }
                    }
                    _ => {
                        exported_other.push(("default", export.span));
                    }
                }
            }
            _ => {}
        }
    }
    
    if !exported_other.is_empty() {
        for (name, span) in &exported_other {
            linter.add_error(
                "one-public-function".to_string(),
                format!("Only functions can be exported. Found non-function export: {}", name),
                *span,
            );
        }
    }
    
    if exported_functions.len() > 1 {
        for (name, span) in &exported_functions[1..] {
            linter.add_error(
                "one-public-function".to_string(),
                format!("Only one function can be exported per file. Found additional export: {}", name),
                *span,
            );
        }
    }
}

pub fn check_no_top_level_side_effects(linter: &mut Linter, program: &Program) {
    for item in &program.body {
        match item {
            Statement::ExpressionStatement(expr_stmt) => {
                match &expr_stmt.expression {
                    Expression::CallExpression(call) => {
                        if !is_iife(call) {
                            linter.add_error(
                                "no-top-level-side-effects".to_string(),
                                "Top-level function calls are not allowed (side effects)".to_string(),
                                expr_stmt.span,
                            );
                        }
                    }
                    Expression::AssignmentExpression(_) => {
                        linter.add_error(
                            "no-top-level-side-effects".to_string(),
                            "Top-level assignments are not allowed (side effects)".to_string(),
                            expr_stmt.span,
                        );
                    }
                    Expression::UpdateExpression(_) => {
                        linter.add_error(
                            "no-top-level-side-effects".to_string(),
                            "Top-level update expressions are not allowed (side effects)".to_string(),
                            expr_stmt.span,
                        );
                    }
                    Expression::NewExpression(_) => {
                        linter.add_error(
                            "no-top-level-side-effects".to_string(),
                            "Top-level new expressions are not allowed (side effects)".to_string(),
                            expr_stmt.span,
                        );
                    }
                    _ => {}
                }
            }
            Statement::ForStatement(_) |
            Statement::ForInStatement(_) |
            Statement::ForOfStatement(_) |
            Statement::WhileStatement(_) |
            Statement::DoWhileStatement(_) => {
                linter.add_error(
                    "no-top-level-side-effects".to_string(),
                    "Top-level loops are not allowed (side effects)".to_string(),
                    item.span(),
                );
            }
            Statement::IfStatement(if_stmt) => {
                if !is_type_guard_only(if_stmt) {
                    linter.add_error(
                        "no-top-level-side-effects".to_string(),
                        "Top-level if statements are not allowed (side effects)".to_string(),
                        if_stmt.span,
                    );
                }
            }
            _ => {}
        }
    }
}

fn is_iife(call: &CallExpression) -> bool {
    match &call.callee {
        Expression::FunctionExpression(_) | 
        Expression::ArrowFunctionExpression(_) => true,
        Expression::ParenthesizedExpression(paren) => {
            matches!(&paren.expression, 
                Expression::FunctionExpression(_) | 
                Expression::ArrowFunctionExpression(_)
            )
        },
        _ => false
    }
}

fn is_type_guard_only(_if_stmt: &IfStatement) -> bool {
    false
}

pub fn check_import_extensions(linter: &mut Linter, program: &Program) {
    struct ImportExtensionChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for ImportExtensionChecker<'a> {
        fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
            let source = import.source.value.as_str();
            
            // Check if it's a relative path import
            if source.starts_with("./") || source.starts_with("../") {
                // Check if it has .ts or .tsx extension
                if !source.ends_with(".ts") && !source.ends_with(".tsx") && !source.ends_with(".js") && !source.ends_with(".jsx") {
                    self.linter.add_error(
                        "import-extensions-required".to_string(),
                        format!("Relative imports must include .ts extension: '{}'", source),
                        import.span,
                    );
                }
            }
        }
        
        fn visit_export_named_declaration(&mut self, export: &ExportNamedDeclaration<'a>) {
            if let Some(source) = &export.source {
                let source_str = source.value.as_str();
                
                // Check if it's a relative path import
                if source_str.starts_with("./") || source_str.starts_with("../") {
                    // Check if it has .ts or .tsx extension
                    if !source_str.ends_with(".ts") && !source_str.ends_with(".tsx") && !source_str.ends_with(".js") && !source_str.ends_with(".jsx") {
                        self.linter.add_error(
                            "import-extensions-required".to_string(),
                            format!("Relative imports must include .ts extension: '{}'", source_str),
                            export.span,
                        );
                    }
                }
            }
        }
        
        fn visit_export_all_declaration(&mut self, export: &ExportAllDeclaration<'a>) {
            let source = export.source.value.as_str();
            
            // Check if it's a relative path import
            if source.starts_with("./") || source.starts_with("../") {
                // Check if it has .ts or .tsx extension
                if !source.ends_with(".ts") && !source.ends_with(".tsx") && !source.ends_with(".js") && !source.ends_with(".jsx") {
                    self.linter.add_error(
                        "import-extensions-required".to_string(),
                        format!("Relative imports must include .ts extension: '{}'", source),
                        export.span,
                    );
                }
            }
        }
    }
    
    let mut checker = ImportExtensionChecker { linter };
    checker.visit_program(program);
}

pub fn check_no_unused_variables(linter: &mut Linter, program: &Program) {
    use std::collections::{HashMap, HashSet};
    
    struct VariableUsageChecker<'a> {
        declared_vars: HashMap<String, oxc_span::Span>,
        used_vars: HashSet<String>,
        imported_vars: HashMap<String, oxc_span::Span>,
        used_imports: HashSet<String>,
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for VariableUsageChecker<'a> {
        fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
            if let Some(specifiers) = &import.specifiers {
                for specifier in specifiers {
                    match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                            let name = spec.local.name.as_str();
                            self.imported_vars.insert(name.to_string(), import.span);
                        }
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                            let name = spec.local.name.as_str();
                            self.imported_vars.insert(name.to_string(), import.span);
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                            let name = spec.local.name.as_str();
                            self.imported_vars.insert(name.to_string(), import.span);
                        }
                    }
                }
            }
        }
        
        fn visit_variable_declaration(&mut self, var_decl: &VariableDeclaration<'a>) {
            for decl in &var_decl.declarations {
                if let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind {
                    self.declared_vars.insert(id.name.to_string(), decl.span);
                }
            }
            walk::walk_variable_declaration(self, var_decl);
        }
        
        fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
            // Add function parameters as declared
            for param in &func.params.items {
                if let BindingPatternKind::BindingIdentifier(id) = &param.pattern.kind {
                    self.declared_vars.insert(id.name.to_string(), param.span);
                }
            }
            walk::walk_function(self, func, flags);
        }
        
        fn visit_identifier_reference(&mut self, id: &IdentifierReference) {
            let name = id.name.as_str();
            if self.declared_vars.contains_key(name) {
                self.used_vars.insert(name.to_string());
            }
            if self.imported_vars.contains_key(name) {
                self.used_imports.insert(name.to_string());
            }
        }
    }
    
    let mut checker = VariableUsageChecker {
        declared_vars: HashMap::new(),
        used_vars: HashSet::new(),
        imported_vars: HashMap::new(),
        used_imports: HashSet::new(),
        linter,
    };
    
    checker.visit_program(program);
    
    // Report unused variables
    for (name, span) in checker.declared_vars {
        if !checker.used_vars.contains(&name) && !name.starts_with('_') {
            checker.linter.add_error(
                "no-unused-variables".to_string(),
                format!("Variable '{}' is declared but never used", name),
                span,
            );
        }
    }
    
    // Report unused imports
    for (name, span) in checker.imported_vars {
        if !checker.used_imports.contains(&name) && !name.starts_with('_') {
            checker.linter.add_error(
                "no-unused-imports".to_string(),
                format!("Import '{}' is declared but never used", name),
                span,
            );
        }
    }
}

pub fn check_no_getters_setters(linter: &mut Linter, program: &Program) {
    struct GetterSetterChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for GetterSetterChecker<'a> {
        fn visit_method_definition(&mut self, method: &MethodDefinition<'a>) {
            match method.kind {
                MethodDefinitionKind::Get => {
                    self.linter.add_error(
                        "no-getters-setters".to_string(),
                        "Getters are not allowed in pure TypeScript subset".to_string(),
                        method.span,
                    );
                }
                MethodDefinitionKind::Set => {
                    self.linter.add_error(
                        "no-getters-setters".to_string(),
                        "Setters are not allowed in pure TypeScript subset".to_string(),
                        method.span,
                    );
                }
                _ => {}
            }
        }
        
        fn visit_property_definition(&mut self, prop: &PropertyDefinition<'a>) {
            // Check for getter/setter in property definitions
            walk::walk_property_definition(self, prop);
        }
        
        fn visit_accessor_property(&mut self, accessor: &AccessorProperty<'a>) {
            self.linter.add_error(
                "no-getters-setters".to_string(),
                "Accessor properties are not allowed in pure TypeScript subset".to_string(),
                accessor.span,
            );
        }
    }
    
    let mut checker = GetterSetterChecker { linter };
    checker.visit_program(program);
}

pub fn check_must_use_return_value(linter: &mut Linter, program: &Program) {
    struct ReturnValueChecker<'a> {
        linter: &'a mut Linter,
        in_statement_position: bool,
    }
    
    impl<'a> Visit<'a> for ReturnValueChecker<'a> {
        fn visit_expression_statement(&mut self, stmt: &ExpressionStatement<'a>) {
            self.in_statement_position = true;
            
            if let Expression::CallExpression(call) = &stmt.expression {
                // Check if this is a known void function (console.log, etc.)
                let is_void_function = match &call.callee {
                    Expression::StaticMemberExpression(member) => {
                        if let Expression::Identifier(obj) = &member.object {
                            let obj_name = obj.name.as_str();
                            let prop_name = member.property.name.as_str();
                            // Allow console methods and similar void functions
                            obj_name == "console" || 
                            (obj_name == "process" && prop_name == "exit") ||
                            (obj_name == "Array" && prop_name == "isArray") // This actually returns a value but checking in statement position
                        } else {
                            false
                        }
                    }
                    _ => false
                };
                
                if !is_void_function && !is_iife(call) {
                    self.linter.add_error(
                        "must-use-return-value".to_string(),
                        "Function return values must be used or assigned".to_string(),
                        stmt.span,
                    );
                }
            }
            
            walk::walk_expression_statement(self, stmt);
            self.in_statement_position = false;
        }
    }
    
    let mut checker = ReturnValueChecker {
        linter,
        in_statement_position: false,
    };
    checker.visit_program(program);
}

pub fn check_no_delete(linter: &mut Linter, program: &Program) {
    struct DeleteChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for DeleteChecker<'a> {
        fn visit_unary_expression(&mut self, expr: &UnaryExpression<'a>) {
            if matches!(expr.operator, UnaryOperator::Delete) {
                self.linter.add_error(
                    "no-delete".to_string(),
                    "Delete operator is not allowed in pure TypeScript subset".to_string(),
                    expr.span,
                );
            }
            walk::walk_unary_expression(self, expr);
        }
    }
    
    let mut checker = DeleteChecker { linter };
    checker.visit_program(program);
}

pub fn check_no_this_in_functions(linter: &mut Linter, program: &Program) {
    struct ThisChecker<'a> {
        linter: &'a mut Linter,
        in_function: bool,
        in_arrow_function: bool,
    }
    
    impl<'a> Visit<'a> for ThisChecker<'a> {
        fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
            let was_in_function = self.in_function;
            self.in_function = true;
            walk::walk_function(self, func, flags);
            self.in_function = was_in_function;
        }
        
        fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
            let was_in_arrow = self.in_arrow_function;
            self.in_arrow_function = true;
            walk::walk_arrow_function_expression(self, arrow);
            self.in_arrow_function = was_in_arrow;
        }
        
        fn visit_this_expression(&mut self, this: &ThisExpression) {
            if self.in_function || self.in_arrow_function {
                self.linter.add_error(
                    "no-this-in-functions".to_string(),
                    "Using 'this' in functions is not allowed in pure TypeScript subset".to_string(),
                    this.span,
                );
            }
        }
    }
    
    let mut checker = ThisChecker {
        linter,
        in_function: false,
        in_arrow_function: false,
    };
    checker.visit_program(program);
}

pub fn check_no_throw(linter: &mut Linter, program: &Program) {
    struct ThrowChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for ThrowChecker<'a> {
        fn visit_throw_statement(&mut self, stmt: &ThrowStatement<'a>) {
            self.linter.add_error(
                "no-throw".to_string(),
                "Throwing exceptions is not allowed. Use Result type from neverthrow instead".to_string(),
                stmt.span,
            );
        }
        
        fn visit_try_statement(&mut self, stmt: &TryStatement<'a>) {
            self.linter.add_error(
                "no-try-catch".to_string(),
                "Try-catch blocks are not allowed. Use Result type from neverthrow instead".to_string(),
                stmt.span,
            );
        }
    }
    
    let mut checker = ThrowChecker { linter };
    checker.visit_program(program);
}

pub fn check_no_foreach(linter: &mut Linter, program: &Program) {
    struct ForEachChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for ForEachChecker<'a> {
        fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
            // Check if this is a .forEach() call
            if let Expression::StaticMemberExpression(member) = &call.callee {
                if member.property.name.as_str() == "forEach" {
                    self.linter.add_error(
                        "no-foreach".to_string(),
                        "forEach is not allowed. Use 'for...of' loop instead".to_string(),
                        call.span,
                    );
                }
            }
            
            walk::walk_call_expression(self, call);
        }
    }
    
    let mut checker = ForEachChecker { linter };
    checker.visit_program(program);
}