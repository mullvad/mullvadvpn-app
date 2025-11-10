import { useSelector } from '../../../redux/store';

export function useAllowLan() {
  const allowLan = useSelector((state) => state.settings.allowLan);
  return allowLan;
}
