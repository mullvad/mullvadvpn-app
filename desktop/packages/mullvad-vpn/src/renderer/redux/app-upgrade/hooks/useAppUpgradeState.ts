import { useSelector } from '../../store';

export const useAppUpgradeState = () => {
  return useSelector((state) => state.appUpgrade.state);
};
