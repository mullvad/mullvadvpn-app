import React from 'react';

import { getDownloadUrl } from '../../../../../../../shared/version';
import { useAppContext } from '../../../../../../context';
import { useVersionSuggestedIsBeta } from '../../../../../../redux/hooks';

export const useOpenDownloadUrl = () => {
  const { suggestedIsBeta } = useVersionSuggestedIsBeta();
  const { openUrl } = useAppContext();
  const openDownloadLink = React.useCallback(async () => {
    await openUrl(getDownloadUrl(suggestedIsBeta));
  }, [openUrl, suggestedIsBeta]);
  return openDownloadLink;
};
