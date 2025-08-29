import { useAppUpgradeEvent } from '../redux/hooks';

export const useAppUpgradeEventType = () => {
  const { event } = useAppUpgradeEvent();

  const appUpgradeEventType = event?.type;

  return appUpgradeEventType;
};
