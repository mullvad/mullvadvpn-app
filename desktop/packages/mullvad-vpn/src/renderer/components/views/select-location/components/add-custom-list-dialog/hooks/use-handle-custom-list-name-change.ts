import React from 'react';

import { useAddCustomListDialogContext } from '../AddCustomListDialogContext';

export function useHandleCustomListNameChange() {
  const {
    form: {
      error,
      setError,
      customListTextField: { handleOnValueChange: onValueChange },
    },
  } = useAddCustomListDialogContext();

  return React.useCallback(
    (newValue: string) => {
      if (error) {
        setError(false);
      }
      onValueChange(newValue);
    },
    [onValueChange, error, setError],
  );
}
