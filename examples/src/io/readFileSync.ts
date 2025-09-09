import { FileNotFoundError } from "../errors/FileNotFoundError.ts";

/**
 * @allow throws
 * Reads a file from the filesystem synchronously.
 */
export function readFileSync(path: string): string {
  // Simulated sync read - throws custom error
  throw new FileNotFoundError(path);
}
