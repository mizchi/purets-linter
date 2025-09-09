// Test file for max-function-params rule

// Error: Function with 3 parameters
function createUser(name: string, email: string, age: number) {
  return { name, email, age };
}

// Error: Function with 4 parameters
function updateUser(id: string, name: string, email: string, age: number) {
  return { id, name, email, age };
}

// Error: Arrow function with too many params
const processData = (input: string, format: string, validate: boolean) => {
  return input;
};

// Error: Method with too many params
const service = {
  sendNotification(userId: string, title: string, body: string, priority: number) {
    console.log(userId, title, body, priority);
  }
};

// OK: Function with 2 parameters (using options object)
interface UserOptions {
  email: string;
  age: number;
  isAdmin?: boolean;
}

function createUserGood(name: string, options: UserOptions) {
  return { name, ...options };
}

// OK: Arrow function with 2 params
const compute = (value: number, multiplier: number) => value * multiplier;

// OK: Single parameter
const double = (n: number) => n * 2;

// OK: No parameters
const getRandom = () => Math.random();

// Error: Even more parameters
function tooMany(
  a: string,
  b: number,
  c: boolean,
  d: any,
  e: string,
  f: number
) {
  return a;
}

// Good pattern: Using destructuring with single object param
function processOptions({ name, email, age }: UserOptions) {
  return { name, email, age };
}