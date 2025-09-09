// Test file for as const, require, and process rules

// OK: as const is allowed
const config = {
  api: "https://api.example.com",
  timeout: 3000
} as const;

const tuple = [1, 2, 3] as const;
const literal = "test" as const;

// Error: Other as casts are not allowed
const num = "123" as any; // Error
const str = 123 as unknown as string; // Error x2
const obj = {} as MyInterface; // Error

// Error: require() is not allowed
const fs = require("fs"); // Error
const path = require("path"); // Error
const myModule = require("./myModule"); // Error

// Error: Global process is not allowed
console.log(process.env.NODE_ENV); // Error
if (process.argv.length > 2) { // Error
  process.exit(1); // Error x2 (process twice)
}

// OK: Import from node:process
import nodeProcess from "node:process";
console.log(nodeProcess.env.HOME); // OK
nodeProcess.exit(0); // OK

// OK: Import from process (also acceptable)
import proc from "process";
proc.stdout.write("Hello"); // OK

// OK: ES6 imports instead of require
import * as fsModule from "fs";
import { readFile } from "fs/promises";

// Error: Still can't use global process even with different name
const env = process.env; // Error

// OK: Destructured import from node:process
import { env as nodeEnv, argv } from "node:process";
console.log(nodeEnv.PATH); // OK
console.log(argv[0]); // OK