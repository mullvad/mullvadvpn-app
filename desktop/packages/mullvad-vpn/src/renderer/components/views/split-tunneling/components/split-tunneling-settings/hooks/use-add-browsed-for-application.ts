import { useCallback } from 'react';

import { useAppContext } from '../../../../../../context';
import { useSplitTunnelingSettingsContext } from '../SplitTunnelingSettingsContext';
import { useAddApplication } from './use-add-application';

export function useAddBrowsedForApplication() {
  const { getSplitTunnelingApplications } = useAppContext();
  const addApplication = useAddApplication();
  const { setApplications } = useSplitTunnelingSettingsContext();

  const addBrowsedForApplication = useCallback(
    async (application: string) => {
      await addApplication(application);
      const { applications } = await getSplitTunnelingApplications();
      setApplications(applications);
    },
    [addApplication, getSplitTunnelingApplications, setApplications],
  );

  return addBrowsedForApplication;
}
