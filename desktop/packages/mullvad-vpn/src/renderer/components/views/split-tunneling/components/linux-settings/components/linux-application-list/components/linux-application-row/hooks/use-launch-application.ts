import { useCallback } from 'react';

import { useLinuxSettingsContext } from '../../../../../LinuxSettingsContext';
import { useLinuxApplicationRowContext } from '../LinuxApplicationRowContext';
import { useHasApplicationWarning } from './use-has-application-warning';

export function useLaunchApplication() {
  const { setShowUnsupportedDialog, splitTunnelingSupported } = useLinuxSettingsContext();
  const { launch, setShowWarningDialog } = useLinuxApplicationRowContext();
  const hasApplicationWarning = useHasApplicationWarning();

  const launchApplication = useCallback(() => {
    if (splitTunnelingSupported === false) {
      setShowUnsupportedDialog(true);
    } else if (hasApplicationWarning) {
      setShowWarningDialog(true);
    } else {
      launch();
    }
  }, [
    hasApplicationWarning,
    launch,
    setShowUnsupportedDialog,
    setShowWarningDialog,
    splitTunnelingSupported,
  ]);

  return launchApplication;
}
