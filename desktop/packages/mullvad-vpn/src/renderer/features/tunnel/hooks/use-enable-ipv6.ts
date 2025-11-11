import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useEnableIpv6() {
  const enableIpv6 = useSelector((state) => state.settings.enableIpv6);
  const { setEnableIpv6 } = useAppContext();
  return { enableIpv6, setEnableIpv6 };
}
