import { describe, test, expect } from "vitest";
import { add, distance } from "@internal/pure-ts-example";

describe("api", () => {
  test("add function should add two numbers", () => {
    expect(add(2, 3)).toBe(5);
    expect(add(-1, 1)).toBe(0);
    expect(add(0, 0)).toBe(0);
  });

  test("distance function should calculate distance between points", () => {
    const point1 = { x: 0, y: 0 };
    const point2 = { x: 3, y: 4 };
    expect(distance(point1, point2)).toBe(5);

    const point3 = { x: 1, y: 1 };
    const point4 = { x: 1, y: 1 };
    expect(distance(point3, point4)).toBe(0);
  });
});
