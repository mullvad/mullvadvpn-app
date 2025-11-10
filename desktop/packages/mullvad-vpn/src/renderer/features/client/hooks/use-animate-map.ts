import { useSelector } from '../../../redux/store';

export function useAnimateMap() {
  const animateMap = useSelector((state) => state.settings.guiSettings.animateMap);
  return animateMap;
}
