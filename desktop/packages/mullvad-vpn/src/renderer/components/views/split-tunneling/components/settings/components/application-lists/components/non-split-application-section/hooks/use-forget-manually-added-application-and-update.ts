import { useCallback } from 'react';

import { type ISplitTunnelingApplication } from '../../../../../../../../../../../shared/application-types';
import { useAppContext } from '../../../../../../../../../../context';
import { useSettingsContext } from '../../../../../SettingsContext';

export function useForgetManuallyAddedApplicationAndUpdate() {
  const { forgetManuallyAddedSplitTunnelingApplication, getSplitTunnelingApplications } =
    useAppContext();
  const { setApplications } = useSettingsContext();

  const forgetManuallyAddedApplicationAndUpdate = useCallback(
    async (application: ISplitTunnelingApplication) => {
      await forgetManuallyAddedSplitTunnelingApplication(application);
      const { applications } = await getSplitTunnelingApplications();
      setApplications(applications);
    },
    [forgetManuallyAddedSplitTunnelingApplication, getSplitTunnelingApplications, setApplications],
  );

  return forgetManuallyAddedApplicationAndUpdate;
}
