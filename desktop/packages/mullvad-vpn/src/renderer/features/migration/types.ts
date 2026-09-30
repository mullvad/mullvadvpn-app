import type { SettingsMigration } from '../../../shared/daemon-rpc-types';

export type LatestSettingsMigrationWithStatus = {
  completed: boolean;
  migration: SettingsMigration;
};
