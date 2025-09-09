import { Result, ok, err } from 'neverthrow';

// This should fail: throwing exceptions
function throwError(): never {
  throw new Error("This is not allowed");
}

// This should fail: try-catch blocks
function tryCatchExample() {
  try {
    doSomething();
  } catch (e) {
    console.error(e);
  }
}

// This should fail: throwing in conditionals
function conditionalThrow(value: number) {
  if (value < 0) {
    throw new Error("Negative value");
  }
  return value;
}

// This should pass: using Result type instead
export function safeOperation(value: number): Result<number, string> {
  if (value < 0) {
    return err("Negative value not allowed");
  }
  return ok(value * 2);
}