// Error: forEach and do-while are not allowed

const items = [1, 2, 3];

// Error: forEach not allowed
items.forEach(item => {
  console.log(item);
});

// Error: do-while not allowed
let i = 0;
do {
  console.log(i);
  i++;
} while (i < 10);