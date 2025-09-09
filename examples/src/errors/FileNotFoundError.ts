/**
 * Error thrown when a file is not found
 */
export class FileNotFoundError extends Error {
  constructor(public readonly path: string) {
    super(`File not found: ${path}`);
    this.name = "FileNotFoundError";
  }
}