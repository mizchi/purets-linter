import { readFile } from "./io/readFile.ts";
import process from "node:process";

/**
 * @allow console
 */
async function main(): Promise<void> {
  console.log("Application started");
  try {
    const data = await readFile("data.txt");
    console.log("File read successfully", data);
  } catch (error) {
    if (Error.isError(error)) {
      console.error("Error reading file:", error.message);
    } else {
      console.error("Unknown error:", error);
      process.exit(1);
    }
  }
  // Main application logic here
}

// This should be allowed in main.ts
main();
