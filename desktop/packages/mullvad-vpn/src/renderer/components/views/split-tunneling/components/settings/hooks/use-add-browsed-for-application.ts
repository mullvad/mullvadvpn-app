import { useCallback } from 'react';

import { useAppContext } from '../../../../../../context';
import { useSettingsContext } from '../SettingsContext';
import { useAddApplication } from './use-add-application';

export function useAddBrowsedForApplication() {
  const { getSplitTunnelingApplications } = useAppContext();
  const addApplication = useAddApplication();
  const { setApplications } = useSettingsContext();

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
