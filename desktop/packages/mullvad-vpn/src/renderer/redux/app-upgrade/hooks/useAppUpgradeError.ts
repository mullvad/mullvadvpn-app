import { useSelector } from '../../store';
import { setAppUpgradeError } from '../actions';

export const useAppUpgradeError = () => {
  return {
    appUpgradeError: useSelector((state) => state.appUpgrade.error),
    setAppUpgradeError,
  };
};
