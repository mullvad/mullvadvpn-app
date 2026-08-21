import { useAppContext } from '../../../context';
import { useVersionCurrent } from '../../../redux/hooks';
import { useSelector } from '../../../redux/store';

export function useSettingsMigrations() {
  const settingsMigrations = useSelector((state) => state.settings.migrations);
  const settingsMigrationsDisplayedForVersion = useSelector(
    (state) => state.settings.guiSettings.settingsMigrationsDisplayedForVersion,
  );
  const settingsMigrationsDismissedForVersion = useSelector(
    (state) => state.settings.guiSettings.settingsMigrationsDismissedForVersion,
  );
  const { clearSettingsMigrations, setDisplayedSettingsMigrations } = useAppContext();
  const { current } = useVersionCurrent();

  const hasSettingsMigrations = settingsMigrations.length > 0;
  const displayedSettingsMigrations = settingsMigrationsDisplayedForVersion === current;
  const dismissedSettingsMigrations = settingsMigrationsDismissedForVersion === current;

  return {
    settingsMigrations,
    hasSettingsMigrations,
    clearSettingsMigrations,
    displayedSettingsMigrations,
    dismissedSettingsMigrations,
    setDisplayedSettingsMigrations,
  };
}
