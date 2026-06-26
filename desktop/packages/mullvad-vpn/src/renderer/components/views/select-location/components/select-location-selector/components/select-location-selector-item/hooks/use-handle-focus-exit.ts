import React from 'react';

import { useSelectLocationSelectorItemContext } from '../SelectLocationSelectorItemContext';
import { useHandleReset } from './use-handle-reset';

export function useHandleFocusExit() {
  const {
    setFocused,
    textField: { value },
  } = useSelectLocationSelectorItemContext();
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
