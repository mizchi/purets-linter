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

## Expected Directory Structure

The linter expects and enforces the following directory structure:

```
project/
├── src/
│   ├── index.ts          # Main entry point (optional)
│   ├── types/            # Type definitions only
│   │   ├── User.ts       # Pure type exports
│   │   └── Config.ts     # Interface definitions
│   ├── pure/             # Pure functions without side effects
│   │   ├── math.ts       # Mathematical operations
│   │   ├── transform.ts  # Data transformations
│   │   └── validate.ts   # Validation logic
│   ├── io/               # I/O operations and side effects
│   │   ├── api.ts        # API calls
│   │   ├── database.ts   # Database operations
│   │   └── file.ts       # File system operations
│   └── lib/              # Library modules
│       └── utils.ts      # Utility functions
├── tests/                # Test files
│   └── *.test.ts
└── package.json
```

### Directory Rules

- **`src/types/`**: Only type definitions, interfaces, and type aliases. No runtime code.
- **`src/pure/`**: Pure functions only. No I/O, no side effects, no global state access, no async.
- **`src/io/`**: All I/O operations and side effects must be isolated here.
- **Files starting with `_`**: Private implementation files, not meant for export.

## Sample Code

### Type Definitions (`src/types/User.ts`)

```typescript
/**
 * User data structure
 * @interface User
 */
export interface User {
  readonly id: string;
  readonly name: string;
  readonly email: string;
  readonly createdAt: Date;
}

// Type aliases must have explicit types
export type UserId = string;
export type UserList = readonly User[];
```

### Pure Functions (`src/pure/userTransform.ts`)

```typescript
import type { User } from "../types/User";

/**
 * Transform user data for display
 * @param {User} user - The user object
 * @returns {string} Formatted user name
 */
export const formatUserName = (user: User): string => {
  return `${user.name} <${user.email}>`;
};

/**
 * Filter active users
 * @param {readonly User[]} users - List of users
 * @param {Date} since - Date threshold
 * @returns {readonly User[]} Filtered users
 */
export const filterActiveUsers = (
  users: readonly User[],
  since: Date
): readonly User[] => {
  return users.filter(user => user.createdAt > since);
};

// Named export matching filename
export { filterActiveUsers as userTransform };
```

### I/O Operations (`src/io/userApi.ts`)

```typescript
import type { User } from "../types/User";

// @allow throws
// @allow timers
// @allow console

/**
 * Fetch user from API
 * @param {string} id - User ID
 * @returns {Promise<User>} User data
 * @throws {Error} When user not found
 */
export const fetchUser = async (id: string): Promise<User> => {
  const response = await fetch(`/api/users/${id}`);
  
  if (!response.ok) {
    throw new Error(`User not found: ${id}`);
  }
  
  return response.json() as Promise<User>;
};

// Named export matching filename
export { fetchUser as userApi };
```

### Main Entry (`src/index.ts`)

```typescript
import type { User } from "./types/User";
import { filterActiveUsers } from "./pure/userTransform";
import { fetchUser } from "./io/userApi";

/**
 * Main application entry point
 */
async function main(): Promise<void> {
  // All async operations must be handled
  const user = await fetchUser("123");
  console.log(user);
}

// Named export matching filename
export { main as index };

// Only execute if this is the main module
if (import.meta.main) {
  main().catch(console.error);
}
```

### Error Handling Pattern

```typescript
// Instead of throwing errors, return Result types
export type Result<T, E = Error> = 
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: E };

/**
 * Parse JSON safely
 * @param {string} json - JSON string
 * @returns {Result<unknown>} Parsed result
 */
export const parseJSON = (json: string): Result<unknown> => {
  try {
    return { ok: true, value: JSON.parse(json) };
  } catch (error) {
    return { ok: false, error: error as Error };
  }
};
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
