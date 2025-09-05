import { useCallback } from 'react';

import { useLinuxSettingsContext } from '../../../LinuxSettingsContext';

export function useHideUnsupportedDialog() {
  const { setShowUnsupportedDialog } = useLinuxSettingsContext();

  const hideUnsupportedDialog = useCallback(() => {
    setShowUnsupportedDialog(false);
  }, [setShowUnsupportedDialog]);

  return hideUnsupportedDialog;
}
