import { useSelector } from '../../../../redux/store';

export const useAppUpgradeError = () => {
  const appUpgradeError = useSelector((state) => state.appUpgrade.error);

  return appUpgradeError;
};
