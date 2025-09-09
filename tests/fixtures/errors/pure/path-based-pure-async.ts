// This should error: async function in pure/
export async function calculate(a: number): Promise<number> {
  return a * 2;
}