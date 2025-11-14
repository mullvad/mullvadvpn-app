import React from 'react';

import { useDialogContext } from '../DialogContext';

export function useAddEventListeners() {
  const { dialogRef, onOpenChange } = useDialogContext();
  React.useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;

    const handleClose = () => onOpenChange?.(false);
    const handleCancel = (e: Event) => {
      e.preventDefault();
      onOpenChange?.(false);
    };

    dialog.addEventListener('close', handleClose);
    dialog.addEventListener('cancel', handleCancel);

    return () => {
      dialog.removeEventListener('close', handleClose);
      dialog.removeEventListener('cancel', handleCancel);
    };
  }, [dialogRef, onOpenChange]);
}
