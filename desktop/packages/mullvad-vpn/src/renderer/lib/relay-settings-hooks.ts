import { TunnelProtocol } from '../../shared/daemon-rpc-types';
import { useSelector } from '../redux/store';

export function useNormalRelaySettings() {
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  return 'normal' in relaySettings ? relaySettings.normal : undefined;
}

// Some features are considered core privacy features and when enabled prevent OpenVPN from being
// used. This hook returns the tunnelprotocol with the exception that it always returns WireGuard
// when any of those features are enabled.
export function useTunnelProtocol(): TunnelProtocol {
  const relaySettings = useNormalRelaySettings();
  const multihop = relaySettings?.wireguard.useMultihop ?? false;
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const quantumResistant = useSelector((state) => state.settings.wireguard.quantumResistant);
  const openVpnDisabled = daita || multihop || quantumResistant;

  return openVpnDisabled ? 'wireguard' : (relaySettings?.tunnelProtocol ?? 'any');
}

export function useNormalBridgeSettings() {
  const bridgeSettings = useSelector((state) => state.settings.bridgeSettings);
  return bridgeSettings.normal;
}
