// Test file for filename-function match rule
// Filename is myFunction.ts, so it should export a function named myFunction

// Error: Function name doesn't match filename
export default function wrongName() {
    return "This should be named myFunction";
}

// OK if this was exported instead:
// export function myFunction() {
//     return "Correct!";
// }