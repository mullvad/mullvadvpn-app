import type { LatestSettingsMigrationWithStatus } from '../types';
import { useLatestCompletedMigration } from './use-latest-completed-migration';
import { useLatestMigration } from './use-latest-migration';

export function useLatestMigrationStatus(): LatestSettingsMigrationWithStatus | undefined {
  const latestMigration = useLatestMigration();
  const latestCompletedMigration = useLatestCompletedMigration();

  if (latestMigration !== undefined) {
    return {
      completed: false,
      migration: latestMigration,
    };
  }

  if (latestCompletedMigration !== undefined) {
    return {
      completed: true,
      migration: latestCompletedMigration,
    };
  }

  return undefined;
}
