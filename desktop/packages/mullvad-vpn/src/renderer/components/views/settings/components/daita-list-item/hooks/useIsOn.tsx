import { useSettingsDaitaEnabled, useSettingsRelaySettings } from '../../../../../../redux/hooks';

export const useIsOn = () => {
  const { daitaEnabled } = useSettingsDaitaEnabled();
  const { relaySettings } = useSettingsRelaySettings();
  const unavailable =
    'normal' in relaySettings ? relaySettings.normal.tunnelProtocol === 'openvpn' : true;
  return daitaEnabled && !unavailable;
};
