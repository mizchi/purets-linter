// Error: Type-related issues

// Error: Empty array needs type
const emptyArray = [];

// Error: Let needs type
let untypedLet;

// Error: Mutable Record not allowed
const record: Record<string, any> = {};

// Error: Interface without extends
interface User {
  name: string;
  age: number;
}

// Error: as cast not allowed
const value = "123" as any;