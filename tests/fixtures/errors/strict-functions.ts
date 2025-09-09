// Test file for filename-function match, JSDoc requirements, and side-effect functions

// Error: Function name doesn't match filename (this file should export 'strict-functions')
export function wrongName() {
    return 42;
}

// Error: No JSDoc comment
export function anotherFunction() {
    return "hello";
}

/**
 * This function has JSDoc
 * @returns A number
 */
export function properFunction(): number {
    return 123;
}

// Error: Direct use of Math.random() in function
function getRandom() {
    return Math.random(); // Error
}

// OK: Math.random() as default parameter
function getRandomOk(randomFn = () => Math.random()) {
    return randomFn();
}

// Error: Direct use of Date.now() in function
function getTimestamp() {
    return Date.now(); // Error
}

// Error: Direct use of new Date() in function
function getCurrentDate() {
    return new Date(); // Error
}

// OK: Date as parameter
function getDateOk(dateProvider = () => new Date()) {
    return dateProvider();
}

// Error: Direct use of setTimeout in function
function delayedAction() {
    setTimeout(() => console.log("hello"), 1000); // Error
}

// OK: setTimeout as parameter
function delayedActionOk(scheduler = setTimeout) {
    scheduler(() => console.log("hello"), 1000);
}

// Error: setInterval in arrow function
const periodicAction = () => {
    setInterval(() => console.log("tick"), 1000); // Error
};

// OK: Side effects outside functions
const globalRandom = Math.random();
const globalDate = new Date();
const globalTimestamp = Date.now();

// Error: Multiple side effects in one function
function multipleViolations() {
    const random = Math.random(); // Error
    const date = new Date(); // Error
    const timestamp = Date.now(); // Error
    
    setTimeout(() => { // Error
        console.log(random, date, timestamp);
    }, 1000);
}

// OK: Using side effects in default parameters
function withDefaults(
    random = Math.random(),
    date = new Date(),
    timer = setTimeout
) {
    return { random, date, timer };
}