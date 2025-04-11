import { useAppUpgradeError } from '../redux/hooks';

export const useHasAppUpgradeError = () => {
  const { error } = useAppUpgradeError();

  const hasAppUpgradeError = error !== undefined;

  return hasAppUpgradeError;
};
