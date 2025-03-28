import { useAppUpgradeEvent } from '../redux/hooks';

export const useHasAppUpgradeEvent = () => {
  const { appUpgradeEvent } = useAppUpgradeEvent();

  const hasAppUpgradeEvent = appUpgradeEvent !== undefined;

  return hasAppUpgradeEvent;
};
