// index.ts should only contain re-exports

export { add } from "./pure/add.ts";
export { distance } from "./pure/distance.ts";
export type { Point } from "./types/Point.ts";
export type { User } from "./types/User.ts";

// This would error - direct exports not allowed in index.ts
// export const version = "1.0.0";