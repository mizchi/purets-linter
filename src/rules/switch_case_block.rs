use oxc_ast::ast::*;
use oxc_ast::visit::walk;
use oxc_ast::Visit;

use crate::Linter;

pub fn check_switch_case_block(linter: &mut Linter, program: &Program) {
    struct SwitchCaseBlockChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for SwitchCaseBlockChecker<'a> {
        fn visit_switch_case(&mut self, case: &SwitchCase<'a>) {
            // Skip default case or cases with no consequent
            if case.consequent.is_empty() {
                walk::walk_switch_case(self, case);
                return;
            }
            
            // Check if the case has a block statement
            let has_block = case.consequent.len() == 1 && 
                matches!(case.consequent.first(), Some(Statement::BlockStatement(_)));
            
            if !has_block {
                // Check if it's just a break statement (which is allowed)
                let only_break = case.consequent.len() == 1 &&
                    matches!(case.consequent.first(), Some(Statement::BreakStatement(_)));
                
                if !only_break {
                    self.linter.add_error(
                        "switch-case-block".to_string(),
                        "Switch case must use block statement: case 'value': { ... }".to_string(),
                        case.span,
                    );
                }
            }
            
            walk::walk_switch_case(self, case);
        }
    }
    
    let mut checker = SwitchCaseBlockChecker { linter };
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
    fn test_case_without_block() {
        let allocator = Allocator::default();
        let source_text = r#"
function badSwitch(value: string) {
  switch (value) {
    case "a":
      console.log("A");
      break;
    case "b":
      const x = 1;
      console.log("B", x);
      break;
    default:
      console.log("default");
  }
}
"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_switch_case_block(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), 3); // All three cases should fail
        assert!(errors.iter().all(|e| e.message.contains("Switch case must use block statement")));
    }

    #[test]
    fn test_case_with_block() {
        let allocator = Allocator::default();
        let source_text = r#"
export function goodSwitch(value: string) {
  switch (value) {
    case "a": {
      console.log("A");
      break;
    }
    case "b": {
      const x = 1;
      console.log("B", x);
      break;
    }
    default: {
      console.log("default");
      break;
    }
  }
}
"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_switch_case_block(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_case_with_only_break() {
        let allocator = Allocator::default();
        let source_text = r#"
export function switchWithBreak(value: string) {
  switch (value) {
    case "skip":
      break;
    case "process": {
      console.log("processing");
      break;
    }
  }
}

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_switch_case_block(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), 0); // Only break statement should be allowed without block
    }

    #[test]
    fn test_mixed_switch_cases() {
        let allocator = Allocator::default();
        let source_text = r#"
function mixedSwitch(value: string) {
  switch (value) {
    case "good": {
      console.log("Good case");
      break;
    }
    case "bad":
      console.log("Bad case");
      break;
    case "skip":
      break;
    default: {
      console.log("Default with block");
      break;
    }
  }
}

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_switch_case_block(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), 1); // Only the "bad" case should fail
        assert!(errors[0].message.contains("Switch case must use block statement"));
    }

    #[test]
    fn test_empty_switch() {
        let allocator = Allocator::default();
        let source_text = r#"
function emptySwitch(value: string) {
  switch (value) {
    // no cases
  }
}

"#;
        let source_type = SourceType::default();
        let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
        let mut linter = Linter::new(Path::new("test-file.ts"), source_text, false);
        
        check_switch_case_block(&mut linter, &program);
        
        let errors = &linter.errors;
        assert_eq!(errors.len(), 0);
    }
}
