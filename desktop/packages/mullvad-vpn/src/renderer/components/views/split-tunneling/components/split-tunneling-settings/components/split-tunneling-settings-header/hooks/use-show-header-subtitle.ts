import { useSplitTunnelingSettingsContext } from '../../../SplitTunnelingSettingsContext';

export function useShowHeaderSubtitle() {
  const { loadingDiskPermissions, splitTunnelingAvailable } = useSplitTunnelingSettingsContext();

  const showHeaderSubtitle = !loadingDiskPermissions && splitTunnelingAvailable;

  return showHeaderSubtitle;
}
