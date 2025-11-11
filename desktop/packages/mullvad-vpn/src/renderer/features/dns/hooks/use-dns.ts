import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useDns() {
  const dns = useSelector((state) => state.settings.dns);
  const { setDnsOptions } = useAppContext();

  return { dns, setDns: setDnsOptions };
}
