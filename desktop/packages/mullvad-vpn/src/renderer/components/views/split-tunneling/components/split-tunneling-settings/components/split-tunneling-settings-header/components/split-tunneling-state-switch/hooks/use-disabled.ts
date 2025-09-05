import { useSelector } from '../../../../../../../../../../redux/store';
import { useSplitTunnelingSettingsContext } from '../../../../../SplitTunnelingSettingsContext';

export function useDisabled() {
  const { loadingDiskPermissions, splitTunnelingAvailable } = useSplitTunnelingSettingsContext();
  const splitTunnelingEnabled = useSelector((state) => state.settings.splitTunneling);

  const disabled = !splitTunnelingEnabled && (!splitTunnelingAvailable || loadingDiskPermissions);

  return disabled;
}
