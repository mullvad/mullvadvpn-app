import React from 'react';

import { useDialogContext } from '../DialogContext';

export function useHandleKeydown() {
  const { open, onOpenChange } = useDialogContext();
  return React.useCallback(
    (e: React.KeyboardEvent<HTMLDialogElement>) => {
      if (e.key === 'Escape') {
        if (!open) return;
        e.preventDefault();
        e.stopPropagation();
        onOpenChange?.(false);
      }
    },
    [open, onOpenChange],
  );
}
