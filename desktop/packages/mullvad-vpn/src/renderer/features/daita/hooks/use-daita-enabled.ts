import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useDaitaEnabled() {
  const daitaEnabled = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const { setEnableDaita } = useAppContext();
  return { daitaEnabled, setEnableDaita };
}
