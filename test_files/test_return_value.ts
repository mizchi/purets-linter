// This should fail: function return value not used
function getValue(): number {
  return 42;
}

getValue(); // Error: return value not used

// This should pass: return value is assigned
const result = getValue();

// This should pass: return value is used in expression
const doubled = getValue() * 2;

// This should pass: console methods are void functions
console.log("Hello");
console.error("Error");

// This should fail: regular function call without using return value
function processData(data: string): string {
  return data.toUpperCase();
}

processData("test"); // Error: return value not used

// This should pass: IIFE is allowed
(() => {
  return "IIFE result";
})();

export function test() {
  // This should pass: return value is used
  return getValue();
}