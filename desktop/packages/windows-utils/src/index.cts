// This module is the CJS entry point for the library.

// The Rust addon.
import * as addon from './load.cjs';

// Use this declaration to assign types to the addon's exports,
// which otherwise by default are `any`.
declare module './load.cjs' {
  function readShortcut(linkPath: string): string | null;
  function pipeIsAdminOwned(pipePath: string): boolean;
}

/**
 * Return path for a shortcut.
 * @param linkPath absolute path to a `.lnk`.
 */
export function readShortcut(linkPath: string): string | null {
  return addon.readShortcut(linkPath);
}

/**
 * Return whether a named pipe is owned by the admin or SYSTEM account.
 * @param pipePath path to a named pipe.
 */
export function pipeIsAdminOwned(pipePath: string): boolean {
  return addon.pipeIsAdminOwned(pipePath);
}
