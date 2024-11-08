// This module is the CJS entry point for the library.

// The Rust addon.
import * as addon from './load.cjs';

// Use this declaration to assign types to the addon's exports,
// which otherwise by default are `any`.
declare module "./load.cjs" {
  function start(cb: () => void): () => void;
}

export function start(cb: () => void): () => void {
  return addon.start(cb);
}
