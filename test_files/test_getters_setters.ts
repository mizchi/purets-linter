// This should fail: getter and setter are not allowed
const obj = {
  _value: 42,
  
  get value() {
    return this._value;
  },
  
  set value(v: number) {
    this._value = v;
  }
};

// This should fail: class with getter/setter
class MyClass {
  private _name: string = "";
  
  get name() {
    return this._name;
  }
  
  set name(value: string) {
    this._name = value;
  }
}

// This should pass: regular methods
export function createCounter() {
  let count = 0;
  return {
    getValue: () => count,
    setValue: (v: number) => { count = v; }
  };
}