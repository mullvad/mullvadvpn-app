import { useCallback } from 'react';

import { useHeaderContext } from '../../../HeaderContext';

export function useShowUnsupportedDialog() {
  const { setShowUnsupportedDialog } = useHeaderContext();

  const showUnsupportedDialog = useCallback(() => {
    setShowUnsupportedDialog(true);
  }, [setShowUnsupportedDialog]);

  return showUnsupportedDialog;
}
