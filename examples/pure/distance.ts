import type { Point } from "../types/Point.ts";

/**
 * Calculates the distance between two points.
 * @param point1 The first point.
 * @param point2 The second point.
 * @returns The distance between the two points.
 */
export function distance(point1: Point, point2: Point): number {
  const dx = point2.x - point1.x;
  const dy = point2.y - point1.y;
  return Math.sqrt(dx * dx + dy * dy);
}
