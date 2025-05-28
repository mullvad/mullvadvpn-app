import { useSelector } from '../../store';

export const useVersionSuggestedUpgrade = () => {
  return { suggestedUpgrade: useSelector((state) => state.version.suggestedUpgrade) };
};
