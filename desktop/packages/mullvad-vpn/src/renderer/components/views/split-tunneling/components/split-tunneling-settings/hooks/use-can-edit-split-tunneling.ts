import { useSelector } from '../../../../../../redux/store';
import { useSplitTunnelingSettingsContext } from '../SplitTunnelingSettingsContext';

export function useCanEditSplitTunneling() {
  const { splitTunnelingAvailable } = useSplitTunnelingSettingsContext();
  const splitTunnelingEnabled = useSelector((state) => state.settings.splitTunneling);

  const canEditSplitTunneling = splitTunnelingEnabled && (splitTunnelingAvailable ?? false);

  return canEditSplitTunneling;
}
