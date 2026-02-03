import React from 'react';

import { useDialogContext } from '../../../DialogContext';

export function useEffectSyncOpen() {
  const { dialogRef, open, onOpenChange } = useDialogContext();
  React.useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog || !dialog.isConnected) return;

    if (open && !dialog.open) {
      dialog.showModal();
    } else if (!open && dialog.open) {
      dialog.requestClose();
    }
  }, [open, onOpenChange, dialogRef]);
}
