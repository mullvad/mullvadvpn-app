import { useAppUpgradeErrorCount } from '../../../../redux/hooks';

export const useErrorCountExceeded = () => {
  const { errorCount } = useAppUpgradeErrorCount();

  const errorCountExceeded = errorCount >= 3;

  return errorCountExceeded;
};
