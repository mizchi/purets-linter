import type { Point } from "../types/Point.ts";

/**
 * Calculates the Euclidean distance between two points.
 * @param p1 First point
 * @param p2 Second point
 * @returns The distance between the two points
 */
export function distance(p1: Point, p2: Point): number {
  const dx = p2.x - p1.x;
  const dy = p2.y - p1.y;
  return Math.sqrt(dx * dx + dy * dy);
}
