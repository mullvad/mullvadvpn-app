import { useSelector } from '../../../redux/store';

export function useShadowsocksCiphers() {
  const shadowsocksCiphers = useSelector((state) => state.settings.shadowsocksCiphers);

  return { shadowsocksCiphers };
}
