import React from 'react';

import { useEditListDialogContext } from '../EditListDialogContext';

export function useHandleCustomListNameChange() {
  const {
    form: {
      error,
      setError,
      customListTextField: { handleOnValueChange: onValueChange },
    },
  } = useEditListDialogContext();

  return React.useCallback(
    (newValue: string) => {
      if (error) {
        setError(false);
      }
      onValueChange(newValue);
    },
    [error, onValueChange, setError],
  );
}
