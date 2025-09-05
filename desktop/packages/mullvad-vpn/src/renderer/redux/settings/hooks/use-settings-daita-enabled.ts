import { useSelector } from '../../store';

export const useSettingsDaitaEnabled = () => {
  return {
    daitaEnabled: useSelector((state) => state.settings.wireguard.daita?.enabled ?? false),
  };
};
