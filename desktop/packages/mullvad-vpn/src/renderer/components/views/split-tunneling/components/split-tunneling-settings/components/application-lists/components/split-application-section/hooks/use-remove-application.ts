import { useCallback } from 'react';

import { type ISplitTunnelingApplication } from '../../../../../../../../../../../shared/application-types';
import { useAppContext } from '../../../../../../../../../../context';
import { useCanEditSplitTunneling } from '../../../../../hooks';

export function useRemoveApplication() {
  const { removeSplitTunnelingApplication, setSplitTunnelingState } = useAppContext();
  const canEditSplitTunneling = useCanEditSplitTunneling();

  const removeApplication = useCallback(
    async (application: ISplitTunnelingApplication) => {
      if (!canEditSplitTunneling) {
        await setSplitTunnelingState(true);
      }
      removeSplitTunnelingApplication(application);
    },
    [removeSplitTunnelingApplication, setSplitTunnelingState, canEditSplitTunneling],
  );

  return removeApplication;
}
