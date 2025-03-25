import { useSelector } from '../../../../redux/store';

export const useVersionSuggestedUpgrade = () => {
  return useSelector((state) => state.version.suggestedUpgrade);
};
