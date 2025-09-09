import fs from "node:fs/promises";
import { FileNotFoundError } from "../errors/FileNotFoundError.ts";

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
