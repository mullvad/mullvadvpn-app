import { useCallback } from 'react';

import { getDownloadUrl } from '../../../../../../../../../../../shared/version';
import { useAppContext } from '../../../../../../../../../../context';
import { useVersionSuggestedIsBeta } from '../../../../../../../../../../redux/hooks';

export const useHandleClick = () => {
  const { suggestedIsBeta } = useVersionSuggestedIsBeta();
  const { openUrl } = useAppContext();

  const downloadUrl = getDownloadUrl(suggestedIsBeta);

  const handleClick = useCallback(async () => {
    await openUrl(downloadUrl);
  }, [downloadUrl, openUrl]);

  return handleClick;
};
