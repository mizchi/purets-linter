use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

/// Initialize a new pure-ts project with the recommended structure
pub fn init_project(path: &Path) -> Result<()> {
    println!("Initializing pure-ts project at: {}", path.display());
    
    // Create directory structure
    create_directory_structure(path)?;
    
    // Generate configuration files
    generate_package_json(path)?;
    generate_tsconfig(path)?;
    generate_gitignore(path)?;
    
    // Generate boilerplate source files
    generate_source_files(path)?;
    
    println!("✓ Project initialized successfully!");
    println!("\nProject structure:");
    println!("  src/");
    println!("    ├── index.ts       # Public API exports");
    println!("    ├── main.ts        # CLI entry point");
    println!("    ├── io/            # I/O operations (async)");
    println!("    ├── pure/          # Pure functions");
    println!("    └── types/         # Type definitions");
    println!("\nNext steps:");
    println!("  cd {}", path.display());
    println!("  pnpm install");
    println!("  pnpm check");
    
    Ok(())
}

fn create_directory_structure(base_path: &Path) -> Result<()> {
    // Create base directory
    fs::create_dir_all(base_path)
        .with_context(|| format!("Failed to create directory: {}", base_path.display()))?;
    
    // Create src and subdirectories
    let src_path = base_path.join("src");
    fs::create_dir_all(&src_path)?;
    fs::create_dir_all(src_path.join("io"))?;
    fs::create_dir_all(src_path.join("pure"))?;
    fs::create_dir_all(src_path.join("types"))?;
    
    Ok(())
}

fn generate_package_json(base_path: &Path) -> Result<()> {
    let package_json = r#"{
  "name": "my-pure-ts-project",
  "version": "0.1.0",
  "type": "module",
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "import": "./dist/index.js"
    }
  },
  "scripts": {
    "check": "pnpm typecheck && pnpm lint:purets",
    "typecheck": "tsc --noEmit",
    "lint:purets": "purets",
    "build": "tsdown",
    "test": "vitest --run",
    "dev": "node src/main.ts"
  },
  "dependencies": {
    "neverthrow": "^8.2.0"
  },
  "devDependencies": {
    "@types/node": "^24.3.1",
    "tsdown": "^0.15.0",
    "typescript": "^5.9.2",
    "vitest": "^3.2.4"
  }
}
"#;
    
    let path = base_path.join("package.json");
    fs::write(&path, package_json)
        .with_context(|| format!("Failed to write package.json: {}", path.display()))?;
    
    Ok(())
}

fn generate_tsconfig(base_path: &Path) -> Result<()> {
    let tsconfig = r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ES2022",
    "moduleResolution": "bundler",
    "lib": ["ES2022"],
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "allowImportingTsExtensions": true,
    "noEmit": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
"#;
    
    let path = base_path.join("tsconfig.json");
    fs::write(&path, tsconfig)
        .with_context(|| format!("Failed to write tsconfig.json: {}", path.display()))?;
    
    Ok(())
}

fn generate_gitignore(base_path: &Path) -> Result<()> {
    let gitignore = r#"node_modules/
dist/
*.log
.DS_Store
.env
coverage/
.vscode/
.idea/
"#;
    
    let path = base_path.join(".gitignore");
    fs::write(&path, gitignore)
        .with_context(|| format!("Failed to write .gitignore: {}", path.display()))?;
    
    Ok(())
}

fn generate_source_files(base_path: &Path) -> Result<()> {
    let src_path = base_path.join("src");
    
    // Generate index.ts
    let index_content = r#"/**
 * Export Policy:
 * - Keep exports minimal. Only export what users actually need.
 * - Do NOT create wrapper functions. Just selectively re-export existing functions.
 * - index.ts is a catalog, not a factory. Don't create new functions here.
 * - When in doubt, don't export. Add exports later when actually needed.
 * - Avoid "export everything" mentality. Be intentional about the public API.
 */

export { add } from "./pure/add.ts";
export { readConfig } from "./io/readConfig.ts";
"#;
    fs::write(src_path.join("index.ts"), index_content)?;
    
    // Generate main.ts
    let main_content = r#"import { readConfig } from "./io/readConfig.ts";
import process from "node:process";

/**
 * @allow console
 */
async function main(): Promise<void> {
  const result = await readConfig();
  
  if (result.isOk()) {
    console.log("Config:", result.value);
  } else {
    console.error("Error:", result.error);
    process.exit(1);
  }
}

main();
"#;
    fs::write(src_path.join("main.ts"), main_content)?;
    
    // Generate pure/add.ts
    let add_content = r#"/**
 * Adds two numbers together
 * @param a - The first number
 * @param b - The second number
 * @returns The sum of a and b
 */
export function add(a: number, b: number): number {
  return a + b;
}
"#;
    fs::write(src_path.join("pure").join("add.ts"), add_content)?;
    
    // Generate types/User.ts
    let user_type = r#"export type User = {
  readonly id: string;
  readonly name: string;
};
"#;
    fs::write(src_path.join("types").join("User.ts"), user_type)?;
    
    // Generate io/readConfig.ts
    let read_config = r#"import fs from "node:fs/promises";
import { Result, ok, err } from "neverthrow";

/**
 * Reads configuration from package.json file
 * @returns A Result containing the parsed JSON or an error
 */
export async function readConfig(): Promise<
  Result<{ [key: string]: unknown }, Error>
> {
  try {
    const data = await fs.readFile("package.json", "utf-8");
    return ok(JSON.parse(data));
  } catch (error) {
    return err(new Error("Failed to read configuration"));
  }
}
"#;
    fs::write(src_path.join("io").join("readConfig.ts"), read_config)?;
    
    Ok(())
}