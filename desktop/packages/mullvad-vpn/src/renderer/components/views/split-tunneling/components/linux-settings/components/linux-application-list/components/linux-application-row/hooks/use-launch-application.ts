import { useCallback } from 'react';

import { useLinuxApplicationRowContext } from '../LinuxApplicationRowContext';
import { useHasApplicationWarning } from './use-has-application-warning';

export function useLaunchApplication() {
  const { application, onSelect, setShowWarningDialog } = useLinuxApplicationRowContext();
  const hasApplicationWarning = useHasApplicationWarning();

  const launchApplication = useCallback(() => {
    if (hasApplicationWarning) {
      setShowWarningDialog(true);
    } else {
      setShowWarningDialog(false);
      onSelect?.(application);
    }
  }, [application, hasApplicationWarning, onSelect, setShowWarningDialog]);

  return launchApplication;
}
