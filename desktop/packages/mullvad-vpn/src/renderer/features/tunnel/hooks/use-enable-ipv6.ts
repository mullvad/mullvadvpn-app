import { useSelector } from '../../../redux/store';

export function useEnableIpv6() {
  const ipv6 = useSelector((state) => state.settings.enableIpv6);
  return ipv6;
}
