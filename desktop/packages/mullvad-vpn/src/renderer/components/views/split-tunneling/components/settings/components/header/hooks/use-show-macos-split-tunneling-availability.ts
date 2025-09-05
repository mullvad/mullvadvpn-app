import { useSettingsContext } from '../../../SettingsContext';

export function useShowMacOsSplitTunnelingAvailability() {
  const { loadingDiskPermissions, splitTunnelingAvailable } = useSettingsContext();

  if (window.env.platform === 'darwin') {
    const needFullDiskPermissions = splitTunnelingAvailable === false;

    const showMacOsSplitTunnelingAvailability = !loadingDiskPermissions && needFullDiskPermissions;

    return showMacOsSplitTunnelingAvailability;
  }

  return false;
}
