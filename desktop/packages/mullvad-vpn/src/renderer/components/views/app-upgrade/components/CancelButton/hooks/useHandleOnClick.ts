import { useCallback } from 'react';

import { useAppContext } from '../../../../../../context';

export const useHandleOnClick = () => {
  const { appUpgradeAbort } = useAppContext();

  const handleOnClick = useCallback(() => {
    appUpgradeAbort();
  }, [appUpgradeAbort]);

  return handleOnClick;
};
