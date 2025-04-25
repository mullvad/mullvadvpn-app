import { useSelector } from '../../store';
import { setLastProgress } from '../actions';

export const useAppUpgradeLastProgress = () => {
  return {
    lastProgress: useSelector((state) => state.appUpgrade.lastProgress),
    setLastProgress,
  };
};
