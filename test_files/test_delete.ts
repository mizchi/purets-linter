// This should fail: delete operator is not allowed
const obj = { foo: 1, bar: 2 };
delete obj.foo;

// This should fail: delete with bracket notation
const data = { a: 1, b: 2 };
delete data['a'];

// This should pass: use destructuring and spread instead
export function removeProperty() {
  const original = { foo: 1, bar: 2, baz: 3 };
  const { foo, ...rest } = original;
  return rest; // returns { bar: 2, baz: 3 }
}