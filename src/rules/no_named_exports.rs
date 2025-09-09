use oxc_ast::ast::*;
use oxc_ast::visit::walk;
use oxc_ast::Visit;

use crate::Linter;

pub fn check_no_named_exports(linter: &mut Linter, program: &Program) {
    struct NamedExportChecker<'a> {
        linter: &'a mut Linter,
    }
    
    impl<'a> Visit<'a> for NamedExportChecker<'a> {
        fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
            // Check if this is an export { foo } style export (not export function/const/type)
            if decl.declaration.is_none() && !decl.specifiers.is_empty() {
                let exported_names: Vec<String> = decl.specifiers.iter().map(|spec| {
                    spec.local.name().to_string()
                }).collect();
                
                self.linter.add_error(
                    "no-named-exports".to_string(),
                    format!(
                        "Named exports '{}' are not allowed. Use direct export: 'export function foo()' or 'export const foo'",
                        exported_names.join(", ")
                    ),
                    decl.span,
                );
            }
            
            walk::walk_export_named_declaration(self, decl);
        }
    }
    
    let mut checker = NamedExportChecker { linter };
    checker.visit_program(program);
}
