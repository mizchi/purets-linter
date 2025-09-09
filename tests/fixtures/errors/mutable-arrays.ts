// Test file for prefer-readonly-array rule

// Error: Array never mutated, should be readonly
const numbers: number[] = [1, 2, 3, 4, 5];
const doubled = numbers.map(n => n * 2);
const filtered = numbers.filter(n => n > 2);
console.log(numbers.length);

// Error: Array literal without type, never mutated
const items = ["a", "b", "c"];
const upperCase = items.map(s => s.toUpperCase());

// Error: Array.from result never mutated
const fromArray = Array.from([1, 2, 3]);
const sum = fromArray.reduce((a, b) => a + b, 0);

// OK: Array is mutated with push
const mutableList: string[] = [];
mutableList.push("item");

// OK: Array is mutated with pop
const stack: number[] = [1, 2, 3];
const last = stack.pop();

// OK: Array element assignment
const grid: number[] = [0, 0, 0];
grid[1] = 5;

// OK: Array is sorted (mutating)
const toSort: number[] = [3, 1, 2];
toSort.sort();

// OK: Already readonly
const readonlyArr: ReadonlyArray<number> = [1, 2, 3];
const mapped = readonlyArr.map(x => x * 2);

// Error: Type annotation but never mutated
const typedArray: Array<string> = ["hello", "world"];
const joined = typedArray.join(", ");