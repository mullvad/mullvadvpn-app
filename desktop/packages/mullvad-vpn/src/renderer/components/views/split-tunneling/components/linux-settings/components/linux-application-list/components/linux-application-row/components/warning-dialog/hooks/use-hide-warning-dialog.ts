import { useCallback } from 'react';

import { useLinuxApplicationRowContext } from '../../../LinuxApplicationRowContext';

export function useHideWarningDialog() {
  const { setShowWarningDialog } = useLinuxApplicationRowContext();

  const hideWarningDialog = useCallback(() => {
    setShowWarningDialog(false);
  }, [setShowWarningDialog]);

  return hideWarningDialog;
}
