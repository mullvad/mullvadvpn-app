import { useSelector } from '../../../../redux/store';

export const useVersionSuggestedUpgrade = () => {
  return { suggestedUpgrade: useSelector((state) => state.version.suggestedUpgrade) };
};
