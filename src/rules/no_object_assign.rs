use oxc_ast::ast::*;
use oxc_ast::visit::walk;
use oxc_ast::Visit;

use crate::Linter;

pub fn check_no_object_assign(linter: &mut Linter, program: &Program) {
    struct ObjectAssignChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for ObjectAssignChecker<'a> {
        fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
            // Check for Object.assign()
            if let Expression::StaticMemberExpression(member) = &call.callee {
                if let Expression::Identifier(obj) = &member.object {
                    if obj.name.as_str() == "Object" && member.property.name.as_str() == "assign" {
                        self.linter.add_error(
                            "no-object-assign".to_string(),
                            "Object.assign is not allowed. Use spread operator (...) instead".to_string(),
                            call.span,
                        );
                    }
                }
            }
            
            walk::walk_call_expression(self, call);
        }
    }
    
    let mut checker = ObjectAssignChecker { linter };
    checker.visit_program(program);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Linter;
    use oxc_allocator::Allocator;
    use oxc_parser::{Parser, ParserReturn};
    use oxc_span::SourceType;
    use std::path::Path;


    #[test]
    fn test_object_assign_usage() {
        let allocator = Allocator::default();
        let source_text = r#"
const target = { a: 1 };
const source = { b: 2 };
const result = Object.assign(target, source);

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_no_object_assign(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Object.assign is not allowed"));
    }

    #[test]
    fn test_object_assign_with_multiple_sources() {
        let allocator = Allocator::default();
        let source_text = r#"
const merged = Object.assign({}, { x: 1 }, { y: 2 }, { z: 3 });

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_no_object_assign(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Object.assign is not allowed"));
    }

    #[test]
    fn test_object_assign_in_function() {
        let allocator = Allocator::default();
        let source_text = r#"
function mergeObjects(a: object, b: object) {
  return Object.assign({}, a, b);
}

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_no_object_assign(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Object.assign is not allowed"));
    }

    #[test]
    fn test_spread_operator_allowed() {
        let allocator = Allocator::default();
        let source_text = r#"
const target2 = { a: 1 };
const source2 = { b: 2 };
const result2 = { ...target2, ...source2 };
const merged2 = { ...{ x: 1 }, ...{ y: 2 }, ...{ z: 3 } };

export function mergeObjectsSpread(a: object, b: object) {
  return { ...a, ...b };
}

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_no_object_assign(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_multiple_object_assign_violations() {
        let allocator = Allocator::default();
        let source_text = r#"
const result1 = Object.assign({}, { a: 1 });
const result2 = Object.assign({}, { b: 2 }, { c: 3 });
function merge(a: object, b: object) {
  return Object.assign({}, a, b);
}

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_no_object_assign(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), 3);
        assert!(errors.iter().all(|e| e.message.contains("Object.assign is not allowed")));
    }
}
