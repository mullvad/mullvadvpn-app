import { useAppContext } from '../../../context';
import { useSelector } from '../../store';

export const useSettingsShowBetaReleases = () => {
  const { setShowBetaReleases } = useAppContext();
  return {
    showBetaReleases: useSelector((state) => state.settings.showBetaReleases),
    setShowBetaReleases,
  };
};
