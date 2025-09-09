// This should error: function export in types/
export function createPoint(): Point {
  return { x: 0, y: 0 };
}

type Point = { x: number; y: number };