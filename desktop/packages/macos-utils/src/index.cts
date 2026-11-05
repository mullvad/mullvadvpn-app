// This module is the CJS entry point for the library.

// The Rust addon.
import * as addon from './load.cjs';

// Use this declaration to assign types to the addon's exports,
// which otherwise by default are `any`.
declare module './load.cjs' {
  function checkLaunchDaemonStatus(): number;
}

/**
 * Check Mullvad VPN LaunchDaemon status.
 * See Apple's documentation on SmAppService for more information.
 */
export function checkLaunchDaemonStatus(): number {
  return addon.checkLaunchDaemonStatus();
}
