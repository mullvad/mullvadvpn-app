import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useUnpinnedWindow() {
  const unpinnedWindow = useSelector((state) => state.settings.guiSettings.unpinnedWindow);
  const { setUnpinnedWindow } = useAppContext();
  return { unpinnedWindow, setUnpinnedWindow };
}
