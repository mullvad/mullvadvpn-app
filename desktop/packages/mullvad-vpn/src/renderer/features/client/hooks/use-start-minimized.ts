import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useStartMinimized() {
  const startMinimized = useSelector((state) => state.settings.guiSettings.startMinimized);
  const { setStartMinimized } = useAppContext();
  return { startMinimized, setStartMinimized };
}
