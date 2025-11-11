import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useSplitTunneling() {
  const splitTunnelingEnabled = useSelector((state) => state.settings.splitTunneling);
  const { setSplitTunnelingState } = useAppContext();

  return { splitTunnelingEnabled, setSplitTunnelingState };
}
