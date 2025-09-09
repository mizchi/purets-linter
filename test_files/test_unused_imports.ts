// This should fail: unused imports
import { foo, bar } from './utils';
import defaultExport from './lib';

// This should pass: import with underscore prefix
import { _ignored } from './helper';

// Only using foo, not bar or defaultExport
export function test() {
  return foo();
}