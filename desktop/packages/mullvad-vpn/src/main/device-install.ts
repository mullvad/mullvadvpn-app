import { isDeviceInstallDisabled as readFromRegistry } from 'windows-utils';

import log from '../shared/logging';

/**
 * Check the `DeviceInstallDisabled` registry value under
 * `HKLM\SYSTEM\CurrentControlSet\Services\DeviceInstall\Parameters`. When it is set to `1`,
 * Windows refuses to install new devices, which prevents the tunnel adapter from being created.
 *
 * Returns undefined when the value is not set or the registry cannot be read.
 */
export function isDeviceInstallDisabled(): boolean | undefined {
  if (process.platform !== 'win32') {
    return undefined;
  }

  try {
    // The registry read is done by the windows-utils native addon, which returns null when the
    // value is missing or the registry cannot be read.
    return readFromRegistry() ?? undefined;
  } catch (error) {
    log.error(
      `Failed to read the DeviceInstallDisabled registry value: ${(error as Error).message}`,
    );
    return undefined;
  }
}
