import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useAutoConnect() {
  const autoConnect = useSelector((state) => state.settings.guiSettings.autoConnect);
  const { setAutoConnect } = useAppContext();
  return { autoConnect, setAutoConnect };
}
