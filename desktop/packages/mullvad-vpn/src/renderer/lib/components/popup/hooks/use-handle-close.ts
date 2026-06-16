import React from 'react';

import { usePopupContext } from '../PopupContext';

export function useHandleClose() {
  const { open, onOpenChange } = usePopupContext();

  return React.useCallback(() => {
    if (!open) return;
    onOpenChange?.(false);
  }, [open, onOpenChange]);
}
