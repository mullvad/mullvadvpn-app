import { useSelector } from '../../../redux/store';

export function useRecents() {
  const recents = useSelector((state) => state.settings.recents);
  return {
    recents,
  };
}
