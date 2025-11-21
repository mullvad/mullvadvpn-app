import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useAnimateMap() {
  const animateMap = useSelector((state) => state.settings.guiSettings.animateMap);
  const { setAnimateMap } = useAppContext();
  return { animateMap, setAnimateMap };
}
