// This should trigger an error (no @allow console)
// purets-expect-error allow-directives
console.log("This expects an error");

// This should NOT trigger an error because of @allow console
/**
 * @allow console
 */
export function testConsole(): void {
  console.log("This is allowed");
}

// This expects a wrong rule name, so it should report unused-expect-error
// purets-expect-error no-such-rule
export function testUnused(): void {
  // This function doesn't trigger no-such-rule
}

// Multiple expected errors
// purets-expect-error allow-directives, unused-vars
const unusedVar = console.log("test");

// Comma-separated rules
// purets-expect-error allow-directives
const result = fetch("https://api.example.com");