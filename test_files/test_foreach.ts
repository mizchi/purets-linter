// This should fail: using forEach
const numbers = [1, 2, 3, 4, 5];
numbers.forEach(n => console.log(n));

// This should fail: forEach with index
const items = ['a', 'b', 'c'];
items.forEach((item, index) => {
  console.log(index, item);
});

// This should fail: nested forEach
const matrix = [[1, 2], [3, 4]];
matrix.forEach(row => {
  row.forEach(cell => console.log(cell));
});

// This should pass: using for...of instead
export function processArray(arr: number[]): number {
  let sum = 0;
  for (const value of arr) {
    sum += value;
  }
  return sum;
}

// This should pass: for...of with entries for index
export function processWithIndex(arr: string[]): void {
  for (const [index, value] of arr.entries()) {
    console.log(index, value);
  }
}

// This should pass: using map for transformation
export function transformArray(arr: number[]): number[] {
  return arr.map(n => n * 2);
}