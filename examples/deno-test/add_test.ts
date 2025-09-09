import { assertEquals } from "https://deno.land/std/testing/asserts.ts";
import { add } from "../pure/add.ts";

Deno.test("add should add two positive numbers", () => {
  assertEquals(add(1, 2), 3);
});

Deno.test("add should add negative numbers", () => {
  assertEquals(add(-1, -2), -3);
});

Deno.test("add should handle zero", () => {
  assertEquals(add(0, 5), 5);
  assertEquals(add(5, 0), 5);
});