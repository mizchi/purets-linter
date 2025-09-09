import { expect } from "@std/expect";
import { add } from "../pure/add.ts";

Deno.test("add should add two positive numbers", () => {
  expect(add(1, 2)).toBe(3);
});

Deno.test("add should add negative numbers", () => {
  expect(add(-1, -2)).toBe(-3);
});

Deno.test("add should handle zero", () => {
  expect(add(0, 5)).toBe(5);
  expect(add(5, 0)).toBe(5);
});