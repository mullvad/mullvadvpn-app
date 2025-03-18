import { AppUpgradeEvent } from '../../../../redux/app-upgrade/actions';
import { useAppUpgradeEvent } from './useAppUpgradeEvent';

export const useAppUpgradeEventType = (): AppUpgradeEvent['type'] | undefined => {
  const appUpgradeEvent = useAppUpgradeEvent();

  const appUpgradeEventType = appUpgradeEvent?.type;

  return appUpgradeEventType;
};
