import { getDownloadUrl } from '../../../../../../../../../../../shared/version';
import { useVersionSuggestedIsBeta } from '../../../../../../../../../../redux/hooks';

export const useDownloadUrl = () => {
  const { suggestedIsBeta } = useVersionSuggestedIsBeta();

  const downloadUrl = getDownloadUrl(suggestedIsBeta);

  return downloadUrl;
};
