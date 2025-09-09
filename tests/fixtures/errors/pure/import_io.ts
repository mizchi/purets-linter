// This should error: pure function importing from io
import { readFile } from "../../io/file";

/**
 * Process data by reading from file
 */
export function import_io(filename: string): string {
  // This violates pure function rules
  return readFile(filename);
}