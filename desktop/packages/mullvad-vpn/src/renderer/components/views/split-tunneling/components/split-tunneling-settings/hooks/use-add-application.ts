import { useCallback } from 'react';

import { type ISplitTunnelingApplication } from '../../../../../../../shared/application-types';
import { useAppContext } from '../../../../../../context';
import { useCanEditSplitTunneling } from './use-can-edit-split-tunneling';

export function useAddApplication() {
  const { addSplitTunnelingApplication, setSplitTunnelingState } = useAppContext();
  const canEditSplitTunneling = useCanEditSplitTunneling();

  const addApplication = useCallback(
    async (application: ISplitTunnelingApplication | string) => {
      if (!canEditSplitTunneling) {
        await setSplitTunnelingState(true);
      }
      await addSplitTunnelingApplication(application);
    },
    [addSplitTunnelingApplication, canEditSplitTunneling, setSplitTunnelingState],
  );

  return addApplication;
}
