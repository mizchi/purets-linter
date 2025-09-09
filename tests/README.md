# Pure TypeScript Linter Test Fixtures

This directory contains test fixtures for the Pure TypeScript linter.

## Directory Structure

```
tests/
├── fixtures/
│   ├── basic/          # Valid Pure TypeScript code samples
│   ├── errors/         # Code with intentional errors for testing
│   └── parallel/       # Files for parallel processing tests
└── README.md
```

## Test Categories

### Basic (Valid Code)
- `correct.ts` - Properly typed functions and types
- `functions.ts` - Pure function examples with Result types
- `test*.ts` - Simple valid function exports

### Errors (Invalid Code)
Test files demonstrating various linting rules:

- `classes.ts` - Classes and abstract classes (not allowed)
- `enums.ts` - Enum declarations (not allowed)
- `throw-delete.ts` - Throw statements and delete operators (not allowed)
- `foreach-dowhile.ts` - forEach and do-while loops (not allowed)
- `exports.ts` - Export violations (named exports, missing types, etc.)
- `types.ts` - Type-related issues (empty arrays, untyped let, mutable Record)
- `http-imports.ts` - HTTP(S) imports (not allowed)

### Parallel
Simple files for testing parallel processing performance.

## Running Tests

```bash
# Test all fixtures
cargo run --release -- tests/fixtures

# Test specific category
cargo run --release -- tests/fixtures/errors

# Test single file
cargo run --release -- tests/fixtures/errors/classes.ts

# Test with parallel jobs
cargo run --release -- tests/fixtures -j 4
```

## Linting Rules Tested

1. **no-classes** - No class declarations
2. **no-enums** - No enum declarations
3. **no-throw** - No throw statements
4. **no-delete** - No delete operators
5. **no-foreach** - No forEach method calls
6. **no-do-while** - No do-while loops
7. **no-named-exports** - Only default exports allowed
8. **export-const-type-required** - Exported consts need type annotations
9. **no-mutable-record** - No mutable Record types
10. **empty-array-requires-type** - Empty arrays need type annotations
11. **let-requires-type** - Let declarations need types
12. **interface-extends-only** - Interfaces must extend other interfaces
13. **no-as-cast** - No type assertions with 'as'
14. **no-http-imports** - No HTTP(S) imports from URLs
15. **import-extensions** - Relative imports must have extensions
16. **no-namespace-imports** - No namespace imports (import * as)