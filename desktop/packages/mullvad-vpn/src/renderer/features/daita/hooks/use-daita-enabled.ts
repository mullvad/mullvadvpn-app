import { useSelector } from '../../../redux/store';

export function useDaitaEnabled() {
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  return daita;
}
