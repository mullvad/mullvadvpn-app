import { useSelector } from '../../store';

export const useSettingsCustomVpn = () => {
  return {
    customVpnConfig: useSelector((state) => state.settings.customVpnConfig),
    customVpnEnabled: useSelector((state) => state.settings.customVpnEnabled),
    customVpnStats: useSelector((state) => state.settings.customVpnStats),
  };
};
