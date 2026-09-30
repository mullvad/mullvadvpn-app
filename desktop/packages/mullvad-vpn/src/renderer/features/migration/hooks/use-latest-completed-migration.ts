import type { SettingsMigration } from '../../../../shared/daemon-rpc-types';
import { useSelector } from '../../../redux/store';
import { keyToSettingsMigration } from '../utils';

export function useLatestCompletedMigration(): SettingsMigration | undefined {
  const completedSettingsMigrations = useSelector(
    (state) => state.settings.guiSettings.completedSettingsMigrations,
  );

  const latestCompletedMigrationKey = completedSettingsMigrations.length
    ? completedSettingsMigrations.at(-1)
    : undefined;
  if (latestCompletedMigrationKey !== undefined) {
    const latestCompletedMigration = keyToSettingsMigration(latestCompletedMigrationKey);
    return latestCompletedMigration?.migration;
  }

  return undefined;
}
