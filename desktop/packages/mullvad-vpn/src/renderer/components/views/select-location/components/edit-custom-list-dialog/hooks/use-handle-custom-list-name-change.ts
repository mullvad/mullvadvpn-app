import React from 'react';

import { useEditCustomListDialogContext } from '../EditCustomListDialogContext';

export function useHandleCustomListNameChange() {
  const {
    form: {
      error,
      setError,
      customListTextField: { handleOnValueChange: onValueChange },
    },
  } = useEditCustomListDialogContext();

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
