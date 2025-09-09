/**
 * Error thrown when a network request fails
 */
export class NetworkError extends Error {
  constructor(
    public readonly url: string,
    public readonly statusCode?: number
  ) {
    super(`Network request failed: ${url}${statusCode ? ` (${statusCode})` : ""}`);
    this.name = "NetworkError";
  }
}