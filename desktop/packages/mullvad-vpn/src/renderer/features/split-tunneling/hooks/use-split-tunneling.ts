import { useSelector } from '../../../redux/store';

export function useSplitTunneling() {
  const splitTunnelingEnabled = useSelector((state) => state.settings.splitTunneling);
  return splitTunnelingEnabled;
}
