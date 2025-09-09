// This should fail: unused variables
const unusedVar = 42;
let anotherUnused = "hello";

// This should pass: variable with underscore prefix
const _ignoredVar = 100;

// This should fail: unused function parameter
export function processData(data: string, unusedParam: number): string {
  return data.toUpperCase();
}