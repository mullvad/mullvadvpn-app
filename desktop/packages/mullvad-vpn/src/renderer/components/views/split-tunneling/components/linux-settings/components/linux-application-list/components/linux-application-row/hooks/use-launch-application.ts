import { useCallback } from 'react';

import { useLinuxSettingsContext } from '../../../../../LinuxSettingsContext';
import { useLinuxApplicationRowContext } from '../LinuxApplicationRowContext';
import { useHasApplicationWarning } from './use-has-application-warning';

export function useLaunchApplication() {
  const { application, onSelect, setShowWarningDialog } = useLinuxApplicationRowContext();
  const { setShowUnsupportedDialog, splitTunnelingSupported } = useLinuxSettingsContext();
  const hasApplicationWarning = useHasApplicationWarning();

  const launchApplication = useCallback(() => {
    if (splitTunnelingSupported === false) {
      setShowUnsupportedDialog(true);
    } else if (hasApplicationWarning) {
      setShowWarningDialog(true);
    } else {
      setShowWarningDialog(false);
      onSelect?.(application);
    }
  }, [
    application,
    hasApplicationWarning,
    onSelect,
    setShowUnsupportedDialog,
    setShowWarningDialog,
    splitTunnelingSupported,
  ]);

  return launchApplication;
}
