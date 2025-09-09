// This should error: non-async function in io/
export function readFileSync(path: string): string {
  return "content";
}

// This should also error
function helperFunction(): void {
  console.log("helper");
}