import { useCallback } from 'react';

import { type ILinuxSplitTunnelingApplication } from '../../../../../../../shared/application-types';
import { useAppContext } from '../../../../../../context';
import { useLinuxSettingsContext } from '../LinuxSettingsContext';

export function useLaunchApplication() {
  const { launchExcludedApplication } = useAppContext();
  const { setBrowseError } = useLinuxSettingsContext();

  const launchApplication = useCallback(
    async (application: ILinuxSplitTunnelingApplication | string) => {
      const result = await launchExcludedApplication(application);
      if ('error' in result) {
        setBrowseError(result.error);
      }
    },
    [launchExcludedApplication, setBrowseError],
  );

  return launchApplication;
}
