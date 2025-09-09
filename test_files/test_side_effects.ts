// This should fail: top-level side effects are not allowed
console.log('hello');

let counter = 0;
counter++;

new Date();

for (let i = 0; i < 10; i++) {
  console.log(i);
}