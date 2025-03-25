import { useAppContext } from '../../../../context';
import { useSelector } from '../../../../redux/store';

export const useSetttingsShowBetaReleases = () => {
  const { setShowBetaReleases } = useAppContext();
  return {
    showBetaReleases: useSelector((state) => state.settings.showBetaReleases),
    setShowBetaReleases,
  };
};
