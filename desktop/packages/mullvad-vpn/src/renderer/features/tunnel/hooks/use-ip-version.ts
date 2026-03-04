import { useSelector } from '../../../redux/store';

export function useIpVersion() {
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const ipVersion = 'normal' in relaySettings ? relaySettings.normal.wireguard.ipVersion : 'any';

  return { ipVersion };
}
