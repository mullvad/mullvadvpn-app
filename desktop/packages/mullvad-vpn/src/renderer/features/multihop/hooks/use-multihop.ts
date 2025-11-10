import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';

export function useMultihop() {
  const normalRelaySettings = useNormalRelaySettings();
  const multihop = normalRelaySettings?.wireguard.useMultihop ?? false;
  return multihop;
}
