import { useSelector } from '../../store';
import { setAppUpgradeError } from '../actions';

export const useAppUpgradeError = () => {
  return {
    error: useSelector((state) => state.appUpgrade.error),
    setAppUpgradeError,
  };
};
