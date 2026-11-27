import { afterAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { isDeviceInstallDisabled } from '../../src/main/device-install';

const readFromRegistryMock = vi.hoisted(() => vi.fn<() => boolean | null>());

vi.mock('windows-utils', () => ({
  isDeviceInstallDisabled: readFromRegistryMock,
}));

describe('Device install disabled registry check', () => {
  const originalPlatform = Object.getOwnPropertyDescriptor(process, 'platform');

  beforeEach(() => {
    readFromRegistryMock.mockReset();
    // The check is only performed on Windows.
    Object.defineProperty(process, 'platform', { value: 'win32', configurable: true });
  });

  afterAll(() => {
    if (originalPlatform) {
      Object.defineProperty(process, 'platform', originalPlatform);
    }
  });

  it('should be disabled when the registry value is set', () => {
    readFromRegistryMock.mockReturnValue(true);
    expect(isDeviceInstallDisabled()).toBe(true);
  });

  it('should not be disabled when the registry value is not set', () => {
    readFromRegistryMock.mockReturnValue(false);
    expect(isDeviceInstallDisabled()).toBe(false);
  });

  it('should be unknown when the registry value is missing', () => {
    readFromRegistryMock.mockReturnValue(null);
    expect(isDeviceInstallDisabled()).toBeUndefined();
  });

  it('should be unknown when the registry cannot be read', () => {
    readFromRegistryMock.mockImplementation(() => {
      throw new Error('Access is denied.');
    });
    expect(isDeviceInstallDisabled()).toBeUndefined();
  });

  it('should not read the registry on other platforms', () => {
    if (originalPlatform) {
      Object.defineProperty(process, 'platform', originalPlatform);
    }
    expect(isDeviceInstallDisabled()).toBeUndefined();
    expect(readFromRegistryMock).not.toHaveBeenCalled();
  });
});
