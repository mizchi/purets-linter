import { describe, it, expect } from "vitest";
import { add } from "../pure/add.ts";

describe("add", () => {
  it("should add two positive numbers", () => {
    expect(add(1, 2)).toBe(3);
  });

  it("should add negative numbers", () => {
    expect(add(-1, -2)).toBe(-3);
  });

  it("should handle zero", () => {
    expect(add(0, 5)).toBe(5);
    expect(add(5, 0)).toBe(5);
  });
});