import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useRecents() {
  const recents = useSelector((state) => state.settings.recents);
  const { setEnabledRecents } = useAppContext();
  const hasRecents = recents !== undefined;

  return {
    recents,
    hasRecents,
    setEnabledRecents,
  };
}
