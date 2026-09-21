import React from 'react';

import { useTextFieldItemContext } from '../TextFieldItemContext';

export function useHandleClearButtonClick() {
  const {
    textField: { inputRef, handleOnValueChange },
  } = useTextFieldItemContext();

  const handleClearButtonClick = React.useCallback(() => {
    handleOnValueChange('');
    inputRef.current?.focus();
  }, [handleOnValueChange, inputRef]);

  return handleClearButtonClick;
}
