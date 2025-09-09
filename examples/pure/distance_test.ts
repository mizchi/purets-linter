import { describe, it, expect } from "vitest";
import { distance } from "./distance.ts";
import type { Point } from "../types/Point.ts";

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