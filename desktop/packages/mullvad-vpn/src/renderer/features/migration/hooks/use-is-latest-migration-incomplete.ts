import { useSelector } from '../../../redux/store';
import type { LatestSettingsMigrationWithStatus } from '../types';

export function useIsLatestMigrationIncomplete(
  latestMigration: LatestSettingsMigrationWithStatus | undefined,
) {
  const migrations = useSelector((state) => state.settings.migrations);

  const completedLatestMigration = latestMigration ? latestMigration?.completed : false;
  const hasIncompleteMigrations = migrations.length > 0;
  return hasIncompleteMigrations && !completedLatestMigration;
}
