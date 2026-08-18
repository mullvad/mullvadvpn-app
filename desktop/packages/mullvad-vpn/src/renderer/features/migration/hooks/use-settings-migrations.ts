import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useSettingsMigrations() {
  const settingsMigrations = useSelector((state) => state.settings.migrations);
  const { clearSettingsMigrations } = useAppContext();
  const hasSettingsMigrations = settingsMigrations.length > 0;

  return {
    settingsMigrations,
    hasSettingsMigrations,
    clearSettingsMigrations,
  };
}
