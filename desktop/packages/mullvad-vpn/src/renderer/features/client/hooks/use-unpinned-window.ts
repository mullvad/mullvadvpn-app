import { useSelector } from '../../../redux/store';

export function useUnpinnedWindow() {
  const unpinnedWindow = useSelector((state) => state.settings.guiSettings.unpinnedWindow);
  return unpinnedWindow;
}
