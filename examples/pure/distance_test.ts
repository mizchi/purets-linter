import { distance } from "./distance.ts";
import type { Point } from "../types/Point.ts";

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
    },
    toBeCloseTo(expected: number, precision: number = 2): void {
      const actualNum = actual as unknown as number;
      const multiplier = Math.pow(10, precision);
      const diff = Math.round((actualNum - expected) * multiplier) / multiplier;
      if (Math.abs(diff) > 0) {
        throw new Error(`Expected ${expected} but got ${actualNum}`);
      }
    }
  };
}

describe("distance", () => {
  it("should calculate distance between two points", () => {
    const point1: Point = { x: 0, y: 0 };
    const point2: Point = { x: 3, y: 4 };
    expect(distance(point1, point2)).toBe(5);
  });

  it("should return 0 for same point", () => {
    const point: Point = { x: 5, y: 5 };
    expect(distance(point, point)).toBe(0);
  });

  it("should handle negative coordinates", () => {
    const point1: Point = { x: -1, y: -1 };
    const point2: Point = { x: 2, y: 3 };
    expect(distance(point1, point2)).toBe(5);
  });
});