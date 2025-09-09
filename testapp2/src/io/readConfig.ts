import fs from "node:fs/promises";
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
