/**
 * Export Policy:
 * - Keep exports minimal. Only export what users actually need.
 * - Do NOT create wrapper functions. Just selectively re-export existing functions.
 * - index.ts is a catalog, not a factory. Don't create new functions here.
 * - When in doubt, don't export. Add exports later when actually needed.
 * - Avoid "export everything" mentality. Be intentional about the public API.
 */

export { add } from "./pure/add.ts";
export { readConfig } from "./io/readConfig.ts";
