import { useSettingsDaitaEnabled, useSettingsRelaySettings } from '../../../../../../redux/hooks';

export const useIsOn = () => {
  const { daitaEnabled } = useSettingsDaitaEnabled();
  const { relaySettings } = useSettingsRelaySettings();
  const unavailable = !('normal' in relaySettings);
  return daitaEnabled && !unavailable;
};
