import React from 'react';

import { useSelectLocationSelectorItemContext } from '../SelectLocationSelectorItemContext';

export function useHandleClearButtonClick() {
  const {
    textField: { inputRef, handleOnValueChange },
  } = useSelectLocationSelectorItemContext();

  const handleClearButtonClick = React.useCallback(() => {
    handleOnValueChange('');
    inputRef.current?.focus();
  }, [handleOnValueChange, inputRef]);

  return handleClearButtonClick;
}
