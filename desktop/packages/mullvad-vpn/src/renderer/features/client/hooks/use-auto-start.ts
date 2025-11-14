import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useAutoStart() {
  const autoStart = useSelector((state) => state.settings.autoStart);
  const { setAutoStart } = useAppContext();
  return { autoStart, setAutoStart };
}
