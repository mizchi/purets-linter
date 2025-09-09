// This should fail: member assignments are not allowed
const obj = { foo: 1 };
obj.foo = 2;
obj['bar'] = 3;