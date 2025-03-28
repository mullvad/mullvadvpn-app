import { useSelector } from '../../store';

export const useVersionSuggestedIsBeta = () => {
  return { suggestedIsBeta: useSelector((state) => state.version.suggestedIsBeta ?? false) };
};
