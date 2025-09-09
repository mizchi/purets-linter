// This should error: function name doesn't match filename
// File is path-based-pure-mismatch.ts but exports wrongName
export function wrongName(a: number): number {
  return a * 2;
}