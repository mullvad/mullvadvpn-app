import { useSelector } from '../../store';

export const useAppUpgradeErrorCount = () => {
  return {
    errorCount: useSelector((state) => state.appUpgrade.errorCount),
  };
};
