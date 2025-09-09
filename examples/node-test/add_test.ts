import { describe, it } from "node:test";
import assert from "node:assert";
import { add } from "../pure/add.ts";

describe("add", () => {
  it("should add two positive numbers", () => {
    assert.strictEqual(add(1, 2), 3);
  });

  it("should add negative numbers", () => {
    assert.strictEqual(add(-1, -2), -3);
  });

  it("should handle zero", () => {
    assert.strictEqual(add(0, 5), 5);
    assert.strictEqual(add(5, 0), 5);
  });
});