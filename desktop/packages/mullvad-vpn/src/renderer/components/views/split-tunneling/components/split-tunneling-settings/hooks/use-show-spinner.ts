import { useSplitTunnelingSettingsContext } from '../SplitTunnelingSettingsContext';

export function useShowSpinner() {
  const { loadingDiskPermissions } = useSplitTunnelingSettingsContext();

  const showSpinner = loadingDiskPermissions;

  return showSpinner;
}
