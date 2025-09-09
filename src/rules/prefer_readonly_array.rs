use oxc_ast::ast::*;
use oxc_ast::Visit;
use oxc_span::Span;
use std::collections::{HashMap, HashSet};

use crate::Linter;

/// Track array variables and their mutating method calls
pub struct ArrayMutabilityVisitor<'a> {
    // Track array variables (name -> span where declared)
    array_variables: HashMap<String, Span>,
    // Track which arrays have been mutated
    mutated_arrays: HashSet<String>,
    // Track which arrays are already readonly
    readonly_arrays: HashSet<String>,
    // Reference to the linter
    linter: &'a mut Linter,
}

// Mutating array methods that change the array in-place
const MUTATING_METHODS: &[&str] = &[
    "push", "pop", "shift", "unshift", "splice", "sort", "reverse", "fill", "copyWithin"
];

impl<'a> ArrayMutabilityVisitor<'a> {
    pub fn new(linter: &'a mut Linter) -> Self {
        Self {
            array_variables: HashMap::new(),
            mutated_arrays: HashSet::new(),
            readonly_arrays: HashSet::new(),
            linter,
        }
    }
    
    pub fn check(&mut self, program: &'a Program<'a>) {
        // First pass: collect array declarations
        self.visit_program(program);
        
        // Report arrays that could be readonly
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
    
    fn is_array_type(type_ann: &TSTypeAnnotation) -> bool {
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
    
    fn is_readonly_array_type(type_ann: &TSTypeAnnotation) -> bool {
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

impl<'a> Visit<'a> for ArrayMutabilityVisitor<'a> {
    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        // Check if it's an array declaration
        if let BindingPatternKind::BindingIdentifier(id) = &decl.id.kind {
            let var_name = id.name.to_string();
            
            // Check if it has an array type annotation
            if let Some(type_ann) = &decl.id.type_annotation {
                if Self::is_array_type(type_ann) {
                    self.array_variables.insert(var_name.clone(), decl.span);
                } else if Self::is_readonly_array_type(type_ann) {
                    self.readonly_arrays.insert(var_name.clone());
                }
            }
            
            // Check if it's initialized with an array literal or Array constructor
            if let Some(init) = &decl.init {
                match init {
                    Expression::ArrayExpression(_) => {
                        // Only track if no type annotation (will infer as mutable array)
                        if decl.id.type_annotation.is_none() {
                            self.array_variables.insert(var_name.clone(), decl.span);
                        }
                    }
                    Expression::NewExpression(new_expr) => {
                        if let Expression::Identifier(id) = &new_expr.callee {
                            if id.name == "Array" {
                                self.array_variables.insert(var_name.clone(), decl.span);
                            }
                        }
                    }
                    Expression::CallExpression(call) => {
                        // Check for Array.from, Array.of, etc.
                        if let Expression::StaticMemberExpression(member) = &call.callee {
                            if let Expression::Identifier(obj) = &member.object {
                                if obj.name == "Array" {
                                    self.array_variables.insert(var_name.clone(), decl.span);
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
    
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        // Check for mutating method calls on tracked arrays
        if let Some(member) = call.callee.as_member_expression() {
            if let MemberExpression::StaticMemberExpression(static_member) = member {
                // Get the object being called on
                if let Expression::Identifier(obj_id) = &static_member.object {
                    let obj_name = obj_id.name.to_string();
                    
                    // Check if this is a tracked array and a mutating method
                    if self.array_variables.contains_key(&obj_name) {
                        let method_name = static_member.property.name.as_str();
                        if MUTATING_METHODS.contains(&method_name) {
                            self.mutated_arrays.insert(obj_name);
                        }
                    }
                }
            }
        }
        
        oxc_ast::visit::walk::walk_call_expression(self, call);
    }
    
    fn visit_assignment_expression(&mut self, expr: &AssignmentExpression<'a>) {
        // Check for array element assignments like arr[0] = value
        if let AssignmentTarget::ComputedMemberExpression(member) = &expr.left {
            if let Expression::Identifier(id) = &member.object {
                let name = id.name.to_string();
                if self.array_variables.contains_key(&name) {
                    self.mutated_arrays.insert(name);
                }
            }
        }
        
        oxc_ast::visit::walk::walk_assignment_expression(self, expr);
    }
}

pub fn check_prefer_readonly_array(linter: &mut Linter, program: &Program) {
    let mut visitor = ArrayMutabilityVisitor::new(linter);
    visitor.check(program);
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
        check_prefer_readonly_array(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.message).collect()
    }

    #[test]
    #[ignore] // This rule is now integrated into CombinedVisitor
    fn test_array_never_mutated() {
        let source = r#"
            const arr: number[] = [1, 2, 3];
            const filtered = arr.filter(x => x > 1);
            const mapped = arr.map(x => x * 2);
            console.log(arr.length);
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Array 'arr' is never mutated"));
        assert!(errors[0].contains("ReadonlyArray"));
    }

    #[test]
    fn test_array_with_push() {
        let source = r#"
            const arr: number[] = [1, 2, 3];
            arr.push(4);
            console.log(arr);
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0); // Should not suggest readonly
    }

    #[test]
    fn test_array_with_pop() {
        let source = r#"
            const stack: string[] = ["a", "b"];
            const last = stack.pop();
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0); // Should not suggest readonly
    }

    #[test]
    fn test_array_element_assignment() {
        let source = r#"
            const arr: number[] = [1, 2, 3];
            arr[0] = 10;
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0); // Should not suggest readonly
    }

    #[test]
    fn test_already_readonly() {
        let source = r#"
            const arr: ReadonlyArray<number> = [1, 2, 3];
            const filtered = arr.filter(x => x > 1);
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0); // Already readonly, no suggestion needed
    }

    #[test]
    fn test_array_literal_no_type() {
        let source = r#"
            const items = [1, 2, 3];
            const doubled = items.map(x => x * 2);
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Array 'items' is never mutated"));
    }

    #[test]
    fn test_array_from() {
        let source = r#"
            const arr = Array.from([1, 2, 3]);
            const sum = arr.reduce((a, b) => a + b, 0);
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Array 'arr' is never mutated"));
    }

    #[test]
    #[ignore] // This rule is now integrated into CombinedVisitor
    fn test_multiple_arrays() {
        let source = r#"
            const mutable: number[] = [1, 2];
            const immutable: number[] = [3, 4];
            mutable.push(3);
            const doubled = immutable.map(x => x * 2);
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Array 'immutable' is never mutated"));
    }

    #[test]
    fn test_sort_mutates() {
        let source = r#"
            const arr: number[] = [3, 1, 2];
            arr.sort();
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0); // sort mutates, should not suggest readonly
    }

    #[test]
    fn test_reverse_mutates() {
        let source = r#"
            const arr: string[] = ["a", "b", "c"];
            arr.reverse();
        "#;
        let errors = parse_and_check(source);
        assert_eq!(errors.len(), 0); // reverse mutates, should not suggest readonly
    }
}