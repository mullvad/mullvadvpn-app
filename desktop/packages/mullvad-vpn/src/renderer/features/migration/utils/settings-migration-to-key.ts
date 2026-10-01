import type { SettingsMigration } from '../../../../shared/daemon-rpc-types';

export function settingsMigrationToKey(migration: SettingsMigration): string {
  switch (migration.type) {
    case 'split-filter':
      return `split-filter-${migration.scenario}`;
    default:
      return migration.type;
  }
}
