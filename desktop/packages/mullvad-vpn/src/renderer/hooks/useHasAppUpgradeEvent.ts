import { useAppUpgradeEvent } from '../redux/hooks';

export const useHasAppUpgradeEvent = () => {
  const { event } = useAppUpgradeEvent();

  const hasAppUpgradeEvent = event !== undefined;

  return hasAppUpgradeEvent;
};
