import React from 'react';

import { useAppContext } from '../../../context';
import { useVersionCurrent } from '../../../redux/hooks';
import { useSelector } from '../../../redux/store';
import { settingsMigrationToKey } from '../utils';
import { useIsLatestMigrationIncomplete } from './use-is-latest-migration-incomplete';
import { useLatestMigrationStatus } from './use-latest-migration-status';

export function useSettingsMigrations() {
  const { clearSettingsMigrations, completeSettingsMigration } = useAppContext();
  const { current } = useVersionCurrent();

  const latestMigrationStatus = useLatestMigrationStatus();
  const isLatestMigrationIncomplete = useIsLatestMigrationIncomplete(latestMigrationStatus);

  const completeAndClearLatestMigration = React.useCallback(async () => {
    if (latestMigrationStatus) {
      completeSettingsMigration(settingsMigrationToKey(latestMigrationStatus.migration, current));
    }
    // Since we only keep track of the latest migration, we clear all previous migrations after completing one.
    await clearSettingsMigrations();
  }, [clearSettingsMigrations, completeSettingsMigration, current, latestMigrationStatus]);

  const notificationDismissedForVersion = useSelector(
    (state) => state.settings.guiSettings.settingsMigrationNotificationDismissedForVersion,
  );
  const dismissedMigrationNotification = notificationDismissedForVersion === current;

  return {
    dismissedMigrationNotification,
    completeAndClearLatestMigration,
    latestMigration: latestMigrationStatus?.migration,
    isLatestMigrationIncomplete,
  };
}
