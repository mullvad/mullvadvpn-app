import React from 'react';

import { useCreateCustomListDialogContext } from '../CreateCustomListDialogContext';

export function useHandleCustomListNameChange() {
  const {
    form: {
      error,
      setError,
      customListTextField: { handleOnValueChange: onValueChange },
    },
  } = useCreateCustomListDialogContext();

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
