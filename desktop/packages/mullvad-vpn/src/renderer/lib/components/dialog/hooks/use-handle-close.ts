import React from 'react';

import { useDialogContext } from '../DialogContext';

export function useHandleClose() {
  const { open, onOpenChange } = useDialogContext();

  return React.useCallback(() => {
    if (!open) return;
    onOpenChange?.(false);
  }, [open, onOpenChange]);
}
