import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useAllowLan() {
  const allowLan = useSelector((state) => state.settings.allowLan);
  const { setAllowLan } = useAppContext();
  return { allowLan, setAllowLan };
}
