import React from 'react';

import { useDialogContext } from '../DialogContext';

export function useHandleClick() {
  const { dialogRef } = useDialogContext();
  return React.useCallback(
    (e: React.MouseEvent<HTMLDialogElement>) => {
      if (e.target === dialogRef.current) {
        e.preventDefault();
        e.stopPropagation();
        dialogRef.current?.close();
      }
    },
    [dialogRef],
  );
}
