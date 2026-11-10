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
  try {
    const { version, type, scenario } = validateKey(key);
    return {
      version,
      migration: {
        type,
        scenario,
      },
    };
  } catch (error) {
    console.error(error);
    return undefined;
  }
}

function validateKey(key: string): {
  version: string;
  type: SettingsMigrationType;
  scenario: SplitFilterMigrationScenario;
} {
  const keyParts = key.split('|');
  if (keyParts.length !== 3) {
    throw new Error(`Invalid key length: ${key}`);
  }
  const [version, type, scenario] = keyParts;
  if (!version || !isSettingsMigrationType(type) || !isSplitFilterMigrationScenario(scenario)) {
    throw new Error(`Invalid key format: ${key}`);
  }
  return { version, type, scenario };
}

function isSettingsMigrationType(type: string): type is SettingsMigrationType {
  return (settingsMigrationTypes as readonly string[]).includes(type);
}

function isSplitFilterMigrationScenario(
  scenario: string,
): scenario is SplitFilterMigrationScenario {
  return (splitFilterMigrationScenarios as readonly string[]).includes(scenario);
}
