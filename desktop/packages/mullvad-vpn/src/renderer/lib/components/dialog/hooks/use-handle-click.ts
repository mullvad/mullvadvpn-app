import React from 'react';

import { useDialogContext } from '../DialogContext';

export function useHandleClick() {
  const { dialogRef } = useDialogContext();
  return React.useCallback(
    (e: React.MouseEvent<HTMLDialogElement>) => {
      const dialog = dialogRef.current;
      if (e.target === dialog) {
        dialog.requestClose();
      }
    },
    [dialogRef],
  );
}
