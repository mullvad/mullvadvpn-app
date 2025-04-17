import { useSelector } from '../../store';

export const useAppUpgradeLastProgress = () => {
  return useSelector((state) => state.appUpgrade.lastProgress);
};
