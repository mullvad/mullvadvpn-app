// This module is the CJS entry point for the library.

// The Rust addon.
import * as addon from './load.cjs';

// Use this declaration to assign types to the addon's exports,
// which otherwise by default are `any`.
declare module './load.cjs' {
  function readShortcut(linkPath: string): string;
}

export function readShortcut(linkPath: string): string {
  return addon.readShortcut(linkPath);
}
