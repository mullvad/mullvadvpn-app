import { useCallback } from 'react';

import { useLinuxSettingsContext } from '../../../LinuxSettingsContext';

export function useShowUnsupportedDialog() {
  const { setShowUnsupportedDialog } = useLinuxSettingsContext();

  const showUnsupportedDialog = useCallback(() => {
    setShowUnsupportedDialog(true);
  }, [setShowUnsupportedDialog]);

  return showUnsupportedDialog;
}
