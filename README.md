# Pure TypeScript Linter

An extremely opinionated TypeScript linter for functional immutable style.

Implements a radical ruleset designed for AI-assisted development, prioritizing mechanical naming and file organization over conventional coding standards.

Focuses on mechanical naming conventions, strict file organization, and code discoverability.

## Installation

```bash
cargo install --git https://github.com/mizchi/purets-linter
```

## Usage

```bash
# Create a new project
purets new my-project

# Check current directory (auto-detects workspace and test runner)
purets

# Check specific directory
purets ./src

# Specify test runner explicitly
purets --test vitest
```

## Features

### Zero Configuration

- **Auto-detection** - Automatically detects monorepo workspaces (pnpm, npm, yarn)
- **Test Runner Detection** - Automatically detects test runner (Vitest, Node.js test, Deno)
- **Gitignore Support** - Respects .gitignore patterns and excludes build artifacts
- **Smart Defaults** - Works out of the box without any configuration

## Core Features

### Strict Ruleset

- **No Classes** - Enforces functional programming style
- **No Enums** - Encourages `const` assertions
- **No Exceptions** - Prohibits `throw` statements, errors must be handled via return values
- **No Dynamic Access** - Prohibits dynamic property access on objects
- **Restricted Side Effects** - Enforces pure functions
- **Path-based Restrictions** - Enforces async/sync based on directory structure

### File Structure Conventions

- `src/pure/` - Pure functions only, no async operations
- `src/types/` - Type definitions only, multiple exports allowed
- `src/io/` - I/O operations, sync-only allowed
- `tests/*.test.ts` or `tests/*_test.ts`

### Naming Conventions

- **Function-Filename Match** - Export function name must match filename
- **Required JSDoc** - Exported functions require documentation
- **Parameter Limits** - Maximum 3 function parameters

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
