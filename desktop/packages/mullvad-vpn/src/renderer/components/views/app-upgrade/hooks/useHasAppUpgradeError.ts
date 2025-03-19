import { useAppUpgradeError } from './useAppUpgradeError';

export const useHasAppUpgradeError = () => {
  const appUpgradeError = useAppUpgradeError();

  const hasAppUpgradeError = appUpgradeError !== undefined;

  return hasAppUpgradeError;
};
