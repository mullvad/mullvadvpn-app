import { useSettingsContext } from '../../../SettingsContext';

export function useShowHeaderSubtitle() {
  const { loadingDiskPermissions, splitTunnelingAvailable } = useSettingsContext();

  const showHeaderSubtitle = !loadingDiskPermissions && splitTunnelingAvailable;

  return showHeaderSubtitle;
}
