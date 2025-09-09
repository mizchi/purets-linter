# pure-ts

An extremely opinionated TypeScript linter. Implements a radical ruleset designed for AI-assisted development, prioritizing mechanical naming and file organization over conventional coding standards.

Focuses on mechanical naming conventions, strict file organization, and code discoverability.

## Features

### Strict Ruleset

- **No Classes** - Enforces functional programming style
- **No Enums** - Encourages `const` assertions
- **No Exceptions** - Prohibits `throw` statements, errors must be handled via return values
- **No Dynamic Access** - Prohibits dynamic property access on objects
- **Restricted Side Effects** - Enforces pure functions
- **Path-based Restrictions** - Enforces async/sync based on directory structure

### File Structure Conventions

- `pure/` - Pure functions only, no async operations
- `types/` - Type definitions only, multiple exports allowed
- `io/` - I/O operations, sync-only allowed

### Naming Conventions

- **Function-Filename Match** - Export function name must match filename
- **Required JSDoc** - Exported functions require documentation
- **Parameter Limits** - Maximum 3 function parameters

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Check file or directory
pure-ts ./src

# Also validate tsconfig.json
pure-ts --validate-tsconfig ./src

# Show detailed errors
pure-ts --verbose ./src

# Compare code (evaluate quality before/after refactoring)
pure-ts compare before.ts after.ts

# Specify test runner
pure-ts --test vitest ./src
```

## Rules

### Basic Restrictions

- `no-classes` - Prohibits class definitions
- `no-enums` - Prohibits enums
- `no-throw` - Prohibits throwing exceptions
- `no-delete` - Prohibits delete operator
- `no-eval` - Prohibits eval function
- `no-foreach` - Prohibits forEach (prefer map/filter/reduce)
- `no-do-while` - Prohibits do-while statements

### Type Safety

- `no-as-cast` - Prohibits as type casting
- `let-requires-type` - let variables require type annotations
- `empty-array-requires-type` - Empty arrays require type annotations
- `prefer-readonly-array` - Array parameters must be readonly
- `no-mutable-record` - Record types must be readonly

### Code Quality

- `no-unused-variables` - Prohibits unused variables
- `no-unused-map` - Prohibits unused map return values
- `must-use-return-value` - Return values must be used
- `catch-error-handling` - catch blocks must handle errors
- `switch-case-block` - switch case statements require blocks

### Import/Export

- `strict-named-exports` - Prohibits named exports
- `no-namespace-imports` - Prohibits namespace imports
- `no-reexports` - Prohibits re-exports
- `import-extensions` - Import paths require extensions
- `no-http-imports` - Prohibits HTTP imports

### Node.js Compatibility

- `no-require` - Prohibits require
- `no-filename-dirname` - Prohibits **filename/**dirname
- `no-global-process` - Prohibits global process
- `node-import-style` - Node.js built-in modules require `node:` prefix
- `forbidden-libraries` - Checks for forbidden libraries

### Function Restrictions

- `max-function-params` - Maximum 3 function parameters
- `no-this-in-functions` - Prohibits this in functions
- `no-side-effect-functions` - Restricts function names with side effects
- `filename-function-match` - Filename must match function name
- `export-requires-jsdoc` - Exported functions require JSDoc
- `jsdoc-param-match` - JSDoc must match parameters

### Path-based Restrictions

- `path-based-restrictions` - Directory-based restrictions
  - `pure/` - Pure functions only, no async
  - `types/` - Type definitions only
  - `io/` - Sync I/O operations only

## Configuration

### tsconfig.json

Recommended settings:

```json
{
  "compilerOptions": {
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "noUncheckedIndexedAccess": true
  }
}
```

### package.json

Dependency validation:

- Checks for forbidden library usage
- Detects unused dependencies

## License

MIT
