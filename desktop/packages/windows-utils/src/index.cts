// This module is the CJS entry point for the library.

// The Rust addon.
import * as addon from './load.cjs';

// Use this declaration to assign types to the addon's exports,
// which otherwise by default are `any`.
declare module './load.cjs' {
  function readShortcut(linkPath: string): string | null;
  function pipeIsAdminOwned(pipePath: string): boolean;
  function isDeviceInstallDisabled(): boolean | null;
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
 * Return whether device installation is disabled on this system, i.e. whether the
 * `DeviceInstallDisabled` registry value under
 * `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\DeviceInstall\Parameters` is set to `1`.
 *
 * Returns null if the value is missing, which means device installation is not disabled, or if
 * the registry cannot be read.
 */
export function isDeviceInstallDisabled(): boolean | null {
  return addon.isDeviceInstallDisabled();
}
