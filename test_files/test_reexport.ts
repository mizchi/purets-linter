// This should fail: re-exports are not allowed
export * from './other';
export { foo } from './another';