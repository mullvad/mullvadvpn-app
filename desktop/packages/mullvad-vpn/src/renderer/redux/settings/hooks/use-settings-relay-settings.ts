import { useSelector } from '../../store';

export const useSettingsRelaySettings = () => {
  return {
    relaySettings: useSelector((state) => state.settings.relaySettings),
  };
};
