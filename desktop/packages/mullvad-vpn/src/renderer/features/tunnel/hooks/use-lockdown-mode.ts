import { useSelector } from '../../../redux/store';

export function useLockdownMode() {
  const lockdownMode = useSelector((state) => state.settings.lockdownMode);
  return lockdownMode;
}
