use oxc_ast::ast::*;
use oxc_ast::visit::walk;
use oxc_ast::Visit;

use crate::Linter;

// Helper function for checking IIFE
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Linter;
    use oxc_allocator::Allocator;
    use oxc_parser::{Parser, ParserReturn};
    use oxc_span::SourceType;
    use std::path::Path;


    #[test]
    fn test_unused_function_return_value() {
        let allocator = Allocator::default();
        let source_text = r#"
function getValue(): number {
  return 42;
}

getValue(); // Error: return value not used

function processData(data: string): string {
  return data.toUpperCase();
}

processData("test"); // Error: return value not used

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_must_use_return_value(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().all(|e| e.message.contains("Function return values must be used")));
    }

    #[test]
    fn test_return_value_used() {
        let allocator = Allocator::default();
        let source_text = r#"
function getValue(): number {
  return 42;
}

const result = getValue();
const doubled = getValue() * 2;

export function test() {
  return getValue();
}

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_must_use_return_value(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_console_methods_allowed() {
        let allocator = Allocator::default();
        let source_text = r#"
console.log("Hello");
console.error("Error");
console.warn("Warning");
"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_must_use_return_value(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_iife_allowed() {
        let allocator = Allocator::default();
        let source_text = r#"
(() => {
  return "IIFE result";
})();

(function() {
  return "Another IIFE";
})();
"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_must_use_return_value(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_mixed_cases() {
        let allocator = Allocator::default();
        let source_text = r#"
function getValue(): number {
  return 42;
}

getValue(); // Should fail
const result = getValue(); // Should pass
console.log("Hello"); // Should pass

(() => {
  return "IIFE";
})(); // Should pass

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_must_use_return_value(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Function return values must be used"));
    }
}
