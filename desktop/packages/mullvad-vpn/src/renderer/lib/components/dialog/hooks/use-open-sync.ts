import React from 'react';

import { useDialogContext } from '../DialogContext';

export function useOpenSync() {
  const { dialogRef, open, onOpenChange } = useDialogContext();
  React.useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog || !dialog.isConnected) return;

    if (open && !dialog.open) {
      dialog.showModal();
      onOpenChange?.(true);
    } else {
      dialog.close();
    }
  }, [open, onOpenChange, dialogRef]);
}
