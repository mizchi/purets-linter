// Pure functions example

type Result<T> = { ok: true; value: T } | { ok: false; error: string };

function divide(a: number, b: number): Result<number> {
  if (b === 0) {
    return { ok: false, error: "Division by zero" };
  }
  return { ok: true, value: a / b };
}

function map<T, U>(arr: readonly T[], fn: (item: T) => U): U[] {
  const result: U[] = [];
  for (const item of arr) {
    result.push(fn(item));
  }
  return result;
}

export default { divide, map };