import { useCallback } from 'react';

import { useLinuxApplicationRowContext } from '../LinuxApplicationRowContext';
import { useHasApplicationWarning } from './use-has-application-warning';

export function useLaunchApplication() {
  const { launch, setShowWarningDialog } = useLinuxApplicationRowContext();
  const hasApplicationWarning = useHasApplicationWarning();

  const launchApplication = useCallback(() => {
    if (hasApplicationWarning) {
      setShowWarningDialog(true);
    } else {
      launch();
    }
  }, [hasApplicationWarning, launch, setShowWarningDialog]);

  return launchApplication;
}
