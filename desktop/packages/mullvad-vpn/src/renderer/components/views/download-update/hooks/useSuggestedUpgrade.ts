import { useSelector } from '../../../../redux/store';

export const useSuggestedUpgrade = () => {
  const suggestedUpgrade = useSelector((state) => state.version.suggestedUpgrade);

  return suggestedUpgrade;
};
