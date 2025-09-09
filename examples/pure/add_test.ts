import { add } from "./add.ts";

/**
 * @allow console
 */
function describe(name: string, fn: () => void): void {
  console.log(`Testing: ${name}`);
  fn();
}

/**
 * @allow console
 */
function it(description: string, fn: () => void): void {
  console.log(`  - ${description}`);
  fn();
}

function expect<T>(actual: T) {
  return {
    toBe(expected: T): void {
      if (actual !== expected) {
        throw new Error(`Expected ${expected} but got ${actual}`);
      }
    }
  };
}

describe("add", () => {
  it("should add two positive numbers", () => {
    expect(add(1, 2)).toBe(3);
  });

  it("should add negative numbers", () => {
    expect(add(-1, -2)).toBe(-3);
  });

  it("should add zero", () => {
    expect(add(0, 5)).toBe(5);
  });
});