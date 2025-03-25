import { useAppContext } from '../../../../context';
import { useSelector } from '../../../../redux/store';

export const useSettingsShowBetaReleases = () => {
  const { setShowBetaReleases } = useAppContext();
  return {
    showBetaReleases: useSelector((state) => state.settings.showBetaReleases),
    setShowBetaReleases,
  };
};
