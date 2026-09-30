import type { SettingsMigration } from '../../../../shared/daemon-rpc-types';
import { useSelector } from '../../../redux/store';

export function useLatestMigration(): SettingsMigration | undefined {
  const migrations = useSelector((state) => state.settings.migrations);
  const latestMigration = migrations.length ? migrations.at(-1) : undefined;

  return latestMigration;
}
