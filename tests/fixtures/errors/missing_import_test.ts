// This should error: test file without importing the tested function
import { someOtherFunction } from "./other.ts";

describe("missing_import", () => {
  it("should test something", () => {
    // Test implementation
  });
});