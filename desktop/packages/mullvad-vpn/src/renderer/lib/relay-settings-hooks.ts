import { useSelector } from '../redux/store';

export function useNormalRelaySettings() {
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  return 'normal' in relaySettings ? relaySettings.normal : undefined;
}
