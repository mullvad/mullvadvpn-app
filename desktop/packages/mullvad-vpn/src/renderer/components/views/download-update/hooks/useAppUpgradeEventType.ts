import { AppUpgradeEventType } from '../../../../redux/download-update/actions';
import { useAppUpgradeEvent } from './useAppUpgradeEvent';

export const useAppUpgradeEventType = (): AppUpgradeEventType | undefined => {
  const appUpgradeEvent = useAppUpgradeEvent();

  const appUpgradeEventType = appUpgradeEvent?.type;

  return appUpgradeEventType;
};
