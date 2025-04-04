import { useMemo } from 'react';

import { useVersionSuggestedUpgrade } from '../../../../../../redux/hooks';

export const useChangelog = () => {
  const { suggestedUpgrade } = useVersionSuggestedUpgrade();

  const changelogMemo = useMemo(() => {
    if (suggestedUpgrade) {
      return suggestedUpgrade.changelog;
    }

    return [];
  }, [suggestedUpgrade]);

  return changelogMemo;
};
