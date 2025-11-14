import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useDaitaDirectOnly() {
  const daitaDirectOnly = useSelector(
    (state) => state.settings.wireguard.daita?.directOnly ?? false,
  );
  const { setDaitaDirectOnly } = useAppContext();
  return { daitaDirectOnly, setDaitaDirectOnly };
}
