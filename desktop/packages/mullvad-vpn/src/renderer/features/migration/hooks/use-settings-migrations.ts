import React from 'react';

import type { SettingsMigration } from '../../../../shared/daemon-rpc-types';
import { useAppContext } from '../../../context';
import { useVersionCurrent } from '../../../redux/hooks';
import { useSelector } from '../../../redux/store';
import { settingsMigrationToKey } from '../utils';

export function useSettingsMigrations() {
  const migrations = useSelector((state) => state.settings.migrations);
  const hasMigrations = migrations.length > 0;

  const { clearSettingsMigrations, completeSettingsMigration } = useAppContext();

  const completeMigration = React.useCallback(
    (migration: SettingsMigration) => {
      completeSettingsMigration(settingsMigrationToKey(migration));
    },
    [completeSettingsMigration],
  );

  const notificationDismissedForVersion = useSelector(
    (state) => state.settings.guiSettings.settingsMigrationNotificationDismissedForVersion,
  );
  const { current } = useVersionCurrent();
  const dismissedMigrationNotification = notificationDismissedForVersion === current;

  const completedSettingsMigrations = useSelector(
    (state) => state.settings.guiSettings.completedSettingsMigrations,
  );
  const latestMigration = migrations.length ? migrations.at(-1) : undefined;
  const completedLatestMigration = latestMigration
    ? completedSettingsMigrations.includes(settingsMigrationToKey(latestMigration))
    : false;

  const hasIncompleteMigration = hasMigrations && !completedLatestMigration;

  return {
    migrations,
    hasMigrations,
    clearSettingsMigrations,
    dismissedMigrationNotification,
    completeMigration,
    latestMigration,
    hasIncompleteMigration,
  };
}
