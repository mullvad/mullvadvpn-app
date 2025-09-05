import { useSelector } from '../../../../../../redux/store';
import { useSettingsContext } from '../SettingsContext';

export function useCanEditSplitTunneling() {
  const { splitTunnelingAvailable } = useSettingsContext();
  const splitTunnelingEnabled = useSelector((state) => state.settings.splitTunneling);

  const canEditSplitTunneling = splitTunnelingEnabled && (splitTunnelingAvailable ?? false);

  return canEditSplitTunneling;
}
