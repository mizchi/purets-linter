// Test file for no-dynamic-access and no-define-property rules

// Error: Dynamic property access
const obj = { foo: 1, bar: 2, baz: 3 };
const key = "foo";
const value1 = obj[key]; // Error: dynamic access

// Error: Bracket notation with string literal
const value2 = obj["bar"]; // Error: should use obj.bar

// Error: Dynamic property assignment  
const dynamicKey = "newProp";
obj[dynamicKey] = 42; // Error: dynamic assignment

// Error: Object.defineProperty
Object.defineProperty(obj, "readOnly", {
  value: 100,
  writable: false,
  enumerable: true,
  configurable: false
});

// Error: Object.defineProperties
Object.defineProperties(obj, {
  prop1: { value: 1, writable: true },
  prop2: { value: 2, writable: false }
});

// OK: Array index access (numeric)
const arr = [1, 2, 3, 4, 5];
const first = arr[0]; // OK: numeric index
arr[2] = 30; // OK: numeric array assignment
const last = arr["4"]; // OK: string that parses to number

// OK: Dot notation
const foo = obj.foo; // OK: dot notation
obj.bar = 20; // OK: dot notation assignment

// Error: Complex dynamic access
const nested = { a: { b: { c: 1 } } };
const path = "b";
const deep = nested.a[path]; // Error: dynamic access

// Error: Computed property with variable
function getProp(o: any, k: string) {
  return o[k]; // Error: dynamic access
}

// OK: Destructuring (alternative to dynamic access)
const { foo: extracted } = obj; // OK

// Error: Using symbols or other expressions
const sym = Symbol("test");
const objWithSym = { [sym]: "value" };
const symValue = objWithSym[sym]; // Error: not numeric

// Error: Template literal as key
const prefix = "pre";
const combined = obj[`${prefix}fix`]; // Error: dynamic access