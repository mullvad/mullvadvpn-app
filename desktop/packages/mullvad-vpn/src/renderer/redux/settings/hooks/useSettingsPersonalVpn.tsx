import { useSelector } from '../../store';

export const useSettingsPersonalVpn = () => {
  return {
    personalVpnConfig: useSelector((state) => state.settings.personalVpnConfig),
    personalVpnEnabled: useSelector((state) => state.settings.personalVpnEnabled),
    personalVpnStats: useSelector((state) => state.settings.personalVpnStats),
  };
};
