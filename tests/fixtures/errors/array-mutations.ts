// Comprehensive test for all mutating array methods

// Arrays that SHOULD trigger prefer-readonly-array (never mutated)
const immutable1: number[] = [1, 2, 3];
const mapped1 = immutable1.map(x => x * 2);

const immutable2: string[] = ["a", "b", "c"];
const filtered = immutable2.filter(x => x !== "b");

const immutable3 = [true, false];
const found = immutable3.find(x => x);

// Arrays that should NOT trigger (uses mutating methods)

// Test push()
const withPush: number[] = [1, 2];
withPush.push(3);

// Test pop()
const withPop: string[] = ["a", "b", "c"];
const popped = withPop.pop();

// Test shift()
const withShift: number[] = [1, 2, 3];
const shifted = withShift.shift();

// Test unshift()
const withUnshift: string[] = ["b", "c"];
withUnshift.unshift("a");

// Test splice()
const withSplice: number[] = [1, 2, 3, 4, 5];
withSplice.splice(2, 1);

// Test sort()
const withSort: number[] = [3, 1, 2];
withSort.sort();

// Test reverse()
const withReverse: string[] = ["a", "b", "c"];
withReverse.reverse();

// Test fill()
const withFill: number[] = new Array(5);
withFill.fill(0);

// Test copyWithin()
const withCopyWithin: number[] = [1, 2, 3, 4, 5];
withCopyWithin.copyWithin(0, 3);

// Test array element assignment (also mutating)
const withAssignment: number[] = [0, 0, 0];
withAssignment[1] = 42;

// Combined: array that uses multiple mutating methods
const multiMutate: string[] = [];
multiMutate.push("first");
multiMutate.unshift("zero");
multiMutate.sort();

// Edge case: array passed to external function (should still be flagged if not mutated locally)
const passedArray: number[] = [1, 2, 3];
console.log(passedArray.length);