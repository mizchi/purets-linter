// Test file for @allow directives

// Error: DOM access without @allow dom
function updateUI() {
    document.getElementById("app"); // Error
    window.location.href = "/"; // Error
}

// Error: Timer functions without @allow timers
function delayed() {
    setTimeout(() => {}, 1000); // Error
    setInterval(() => {}, 1000); // Error
}

// Error: Console without @allow console
function debug() {
    console.log("debug"); // Error
}

// Error: Fetch without @allow net
async function getData() {
    const res = await fetch("/api"); // Error
    return res;
}

// Error: DOM types without @allow dom
function handleEvent(event: MouseEvent): HTMLElement { // Error x2
    return document.body; // Error
}

// Error: Network types without @allow net
async function makeRequest(init: RequestInit): Promise<Response> { // Error x2
    return fetch("/api", init); // Error
}

/**
 * OK: Function with @allow dom
 * @allow dom
 */
function withDomAccess() {
    document.getElementById("app"); // OK
    window.location.href = "/"; // OK
}

/**
 * OK: Function with @allow timers
 * @allow timers
 */
function withTimers() {
    setTimeout(() => {}, 1000); // OK
    setInterval(() => {}, 1000); // OK
}

/**
 * OK: Function with multiple allows
 * @allow dom
 * @allow net
 * @allow console
 * @allow timers
 */
async function fullAccess() {
    console.log("Starting..."); // OK
    const data = await fetch("/api"); // OK
    document.body.innerHTML = await data.text(); // OK
    setTimeout(() => console.log("Done"), 1000); // OK
}

/**
 * OK: Function with @allow mutations (allows Date.now, Math.random, etc)
 * @allow mutations
 */
function withMutations() {
    const now = Date.now(); // OK with @allow mutations
    const random = Math.random(); // OK with @allow mutations
    const date = new Date(); // OK with @allow mutations
    return { now, random, date };
}

// Error: Process without @allow process or import
function useProcess() {
    console.log(process.env.NODE_ENV); // Error (both console and process)
}

/**
 * OK: With @allow process
 * @allow process
 * @allow console
 */
function withProcess() {
    console.log(process.env.NODE_ENV); // OK
}