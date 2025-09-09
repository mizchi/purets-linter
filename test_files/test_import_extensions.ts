// This should fail: relative imports without .ts extension
import { foo } from './utils';
import { bar } from '../lib/helper';

// This should pass: relative imports with .ts extension
import { baz } from './other.ts';
import { qux } from '../shared/common.ts';

// This should pass: non-relative imports don't need extensions
import { createElement } from 'react';

// This should fail: re-export without extension
export { something } from './another';

export function test() {
  return foo();
}