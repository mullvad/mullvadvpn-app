import { useSelector } from '../../../../../../../../../../redux/store';
import { useSettingsContext } from '../../../../../SettingsContext';

export function useDisabled() {
  const { loadingDiskPermissions, splitTunnelingAvailable } = useSettingsContext();
  const splitTunnelingEnabled = useSelector((state) => state.settings.splitTunneling);

  const disabled = !splitTunnelingEnabled && (!splitTunnelingAvailable || loadingDiskPermissions);

  return disabled;
}
