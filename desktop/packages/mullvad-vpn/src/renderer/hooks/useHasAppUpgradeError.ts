import { useAppUpgradeError } from '../redux/hooks';

export const useHasAppUpgradeError = () => {
  const { appUpgradeError } = useAppUpgradeError();

  const hasAppUpgradeError = appUpgradeError !== undefined;

  return hasAppUpgradeError;
};
