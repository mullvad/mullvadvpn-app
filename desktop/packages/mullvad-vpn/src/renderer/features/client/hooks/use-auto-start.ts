import { useSelector } from '../../../redux/store';

export function useAutoStart() {
  const autoStart = useSelector((state) => state.settings.autoStart);
  return autoStart;
}
