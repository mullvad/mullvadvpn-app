import { useSelector } from '../../store';

export const useSettingsSplitTunnelingSupported = () => {
  return {
    splitTunnelingSupported: useSelector((state) => state.settings.splitTunnelingSupported),
  };
};
