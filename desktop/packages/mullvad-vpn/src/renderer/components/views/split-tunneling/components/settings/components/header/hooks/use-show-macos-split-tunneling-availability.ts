import { useSettingsContext } from '../../../SettingsContext';

export function useShowMacOsSplitTunnelingAvailability() {
  const { loadingDiskPermissions, splitTunnelingAvailable } = useSettingsContext();

  if (window.env.platform === 'darwin') {
    const needFullDiskPermissions = splitTunnelingAvailable === false;

    return !loadingDiskPermissions && needFullDiskPermissions;
  }

  return false;
}
