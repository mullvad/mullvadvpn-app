import { useAppUpgradeEvent } from '../redux/hooks';

export const useAppUpgradeEventType = () => {
  const { appUpgradeEvent } = useAppUpgradeEvent();

  const appUpgradeEventType = appUpgradeEvent?.type;

  return appUpgradeEventType;
};
