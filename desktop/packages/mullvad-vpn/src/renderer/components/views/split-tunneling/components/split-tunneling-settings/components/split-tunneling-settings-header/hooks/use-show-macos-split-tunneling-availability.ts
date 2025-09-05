import { useSplitTunnelingSettingsContext } from '../../../SplitTunnelingSettingsContext';

export function useShowMacOsSplitTunnelingAvailability() {
  const { loadingDiskPermissions, splitTunnelingAvailable } = useSplitTunnelingSettingsContext();

  if (window.env.platform === 'darwin') {
    const needFullDiskPermissions = splitTunnelingAvailable === false;

    const showMacOsSplitTunnelingAvailability = !loadingDiskPermissions && needFullDiskPermissions;

    return showMacOsSplitTunnelingAvailability;
  }

  return false;
}
