import { useSelector } from '../../../redux/store';

export function useAutoConnect() {
  const autoConnect = useSelector((state) => state.settings.guiSettings.autoConnect);
  return autoConnect;
}
