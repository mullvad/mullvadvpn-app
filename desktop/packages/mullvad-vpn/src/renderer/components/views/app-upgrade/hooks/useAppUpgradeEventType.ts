import { useAppUpgradeEvent } from './useAppUpgradeEvent';

export const useAppUpgradeEventType = () => {
  const appUpgradeEvent = useAppUpgradeEvent();

  const appUpgradeEventType = appUpgradeEvent?.type;

  return appUpgradeEventType;
};
