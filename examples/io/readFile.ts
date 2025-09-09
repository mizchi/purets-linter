import fs from "node:fs/promises";
import { FileNotFoundError } from "./errors/FileNotFoundError.ts";

/**
 * @allow throws
 * Reads a file from the filesystem.
 * This can be either sync or async in io/
 */
export function readFileSync(path: string): string {
  // Simulated sync read - throws custom error
  throw new FileNotFoundError(path);
}

/**
 * @allow throws
 * Async version of file reading
 */
export async function readFile(path: string): Promise<string> {
  try {
    return await fs.readFile(path, "utf-8");
  } catch (error) {
    throw new FileNotFoundError(path);
  }
}