import { readConfig } from "./io/readConfig.ts";
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
