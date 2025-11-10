import { useSplitTunneling } from '../../../../../../../../../../features/split-tunneling/hooks';
import { useSplitTunnelingSettingsContext } from '../../../../../SplitTunnelingSettingsContext';

export function useDisabled() {
  const { loadingDiskPermissions, splitTunnelingAvailable } = useSplitTunnelingSettingsContext();
  const splitTunnelingEnabled = useSplitTunneling();

  const disabled = !splitTunnelingEnabled && (!splitTunnelingAvailable || loadingDiskPermissions);

  return disabled;
}
