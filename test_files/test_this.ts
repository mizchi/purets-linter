// This should fail: using this in a regular function
function regularFunction() {
  return this.value;
}

// This should fail: using this in an arrow function
const arrowFunction = () => {
  return this.value;
};

// This should fail: using this in a method
const objectWithMethod = {
  value: 42,
  getValue: function() {
    return this.value;
  },
  getValueArrow: () => {
    return this.value;
  }
};

// This should pass: not using this
export function pureFunction(value: number) {
  return value * 2;
}