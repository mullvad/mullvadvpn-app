import {
  type SettingsMigration,
  type SettingsMigrationType,
  settingsMigrationTypes,
  type SplitFilterMigrationScenario,
  splitFilterMigrationScenarios,
} from '../../../../shared/daemon-rpc-types';

export function settingsMigrationToKey(
  migration: SettingsMigration,
  currentVersion: string,
): string {
  switch (migration.type) {
    case 'split-filter':
      return `${currentVersion}|split-filter|${migration.scenario}`;
    default:
      return `${currentVersion}|${migration.type}`;
  }
}

export function keyToSettingsMigration(key: string):
  | {
      version: string;
      migration: SettingsMigration;
    }
  | undefined {
  const [version, type, scenario] = key.split('|');
  if (
    type &&
    isSettingsMigrationType(type) &&
    scenario &&
    isSplitFilterMigrationScenario(scenario)
  ) {
    return {
      version,
      migration: {
        type: 'split-filter',
        scenario,
      },
    };
  }
  console.error(`Invalid settings migration key: ${key}`);
  return undefined;
}

function isSettingsMigrationType(type: string): type is SettingsMigrationType {
  return (settingsMigrationTypes as readonly string[]).includes(type);
}

function isSplitFilterMigrationScenario(
  scenario: string,
): scenario is SplitFilterMigrationScenario {
  return (splitFilterMigrationScenarios as readonly string[]).includes(scenario);
}
