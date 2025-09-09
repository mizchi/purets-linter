use oxc::ast::ast::*;
use oxc::ast_visit::walk;
use oxc::ast_visit::Visit;

use crate::Linter;

pub fn check_no_foreach(linter: &mut Linter, program: &Program) {
    struct ForEachChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for ForEachChecker<'a> {
        fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
            // Check if this is a .forEach() call
            if let Expression::StaticMemberExpression(member) = &call.callee {
                if member.property.name == "forEach" {
                    self.linter.add_error(
                        "no-foreach".to_string(),
                        "forEach is not allowed in pure TypeScript subset. Use for-of loop instead".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Linter;
    use oxc::allocator::Allocator;
    use oxc::parser::Parser;
    use oxc::span::SourceType;
    use std::path::Path;

    fn parse_and_check(source: &str) -> Vec<String> {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let ret = Parser::new(&allocator, source, source_type).parse();
        
        let mut linter = Linter::new(Path::new("test-file.ts"), source, false);
        check_no_foreach(&mut linter, &ret.program);
        
        linter.errors.into_iter().map(|e| e.rule).collect()
    }

    #[test]
    fn test_array_foreach() {
        let source = r#"
            const arr = [1, 2, 3];
            arr.forEach((item) => {
                console.log(item);
            });
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-foreach".to_string()));
    }

    #[test]
    fn test_foreach_with_index() {
        let source = r#"
            const items = ['a', 'b', 'c'];
            items.forEach((item, index) => {
                console.log(index, item);
            });
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-foreach".to_string()));
    }

    #[test]
    fn test_for_of_allowed() {
        let source = r#"
            const arr = [1, 2, 3];
            for (const item of arr) {
                console.log(item);
            }
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_map_allowed() {
        let source = r#"
            const arr = [1, 2, 3];
            const doubled = arr.map(x => x * 2);
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_chained_foreach() {
        let source = r#"
            [1, 2, 3]
                .filter(x => x > 1)
                .forEach(x => console.log(x));
        "#;
        
        let errors = parse_and_check(source);
        assert!(errors.contains(&"no-foreach".to_string()));
    }
}
