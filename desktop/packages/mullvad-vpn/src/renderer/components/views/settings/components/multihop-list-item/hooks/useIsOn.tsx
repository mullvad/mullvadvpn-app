import { useSettingsRelaySettings } from '../../../../../../redux/hooks';

export const useIsOn = () => {
  const { relaySettings } = useSettingsRelaySettings();
  const multihopEnabled =
    'normal' in relaySettings ? relaySettings.normal.wireguard.useMultihop : false;
  const unavailable =
    'normal' in relaySettings ? relaySettings.normal.tunnelProtocol === 'openvpn' : true;
  return multihopEnabled && !unavailable;
};
