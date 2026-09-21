import React from 'react';

import { useTextFieldItemContext } from '../TextFieldItemContext';
import { useHandleReset } from './use-handle-reset';

export function useHandleFocusExit() {
  const {
    setFocused,
    textField: { value },
  } = useTextFieldItemContext();
  const handleReset = useHandleReset();

  const handleFocusExit = React.useCallback(() => {
    const shouldReset = value.length === 0;
    if (shouldReset) {
      handleReset();
    }

    setFocused(false);
  }, [handleReset, setFocused, value]);

  return handleFocusExit;
}
