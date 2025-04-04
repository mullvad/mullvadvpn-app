import { useAppUpgradeEventType } from './useAppUpgradeEventType';

export const useHasAppUpgradeInitiated = () => {
  const appUpgradeEventType = useAppUpgradeEventType();

  const hasAppUpgradeInitiated = appUpgradeEventType !== undefined;

  return hasAppUpgradeInitiated;
};
