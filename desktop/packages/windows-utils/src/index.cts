// This module is the CJS entry point for the library.

// The Rust addon.
import * as addon from './load.cjs';

// Use this declaration to assign types to the addon's exports,
// which otherwise by default are `any`.
declare module './load.cjs' {
  function readShortcut(linkPath: string): string | null;
  function pipeIsAdminOwned(pipePath: string): boolean;
  function vmwareBorkedMyMachine(): boolean;
  function unborkMyMachineFromVmware(): void;
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

/**
 * Returns true if the registry key `HKLM\SOFTWARE\Classes\CLSID{3d09c1ca-2bcc-40b7-b9bb-3f3ec143a87b}` exists,
 * which is known to cause conflicts when creating tun devices.
 */
export function vmwareBorkedMyMachine(): boolean {
  return addon.vmwareBorkedMyMachine();
}

/**
 * Try to remove lingering VMWare artifacts that may cause conflicts with Mullvad VPN.
 *
 * This function may throw an error if editing the Windows registry fails.
 */
export function unborkMyMachineFromVmware(): void {
  return addon.unborkMyMachineFromVmware();
}
