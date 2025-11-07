import { useSelector } from '../../../redux/store';

export function useDaitaDirectOnly() {
  const directOnly = useSelector((state) => state.settings.wireguard.daita?.directOnly ?? false);
  return directOnly;
}
