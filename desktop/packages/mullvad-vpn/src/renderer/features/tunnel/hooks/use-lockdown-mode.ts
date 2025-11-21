import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useLockdownMode() {
  const lockdownMode = useSelector((state) => state.settings.lockdownMode);
  const { setLockdownMode } = useAppContext();
  return { lockdownMode, setLockdownMode };
}
