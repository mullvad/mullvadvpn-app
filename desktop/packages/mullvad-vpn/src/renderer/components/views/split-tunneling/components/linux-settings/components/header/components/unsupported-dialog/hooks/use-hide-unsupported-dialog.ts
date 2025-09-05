import { useCallback } from 'react';

import { useHeaderContext } from '../../../HeaderContext';

export function useHideUnsupportedDialog() {
  const { setShowUnsupportedDialog } = useHeaderContext();

  const hideUnsupportedDialog = useCallback(() => {
    setShowUnsupportedDialog(false);
  }, [setShowUnsupportedDialog]);

  return hideUnsupportedDialog;
}
