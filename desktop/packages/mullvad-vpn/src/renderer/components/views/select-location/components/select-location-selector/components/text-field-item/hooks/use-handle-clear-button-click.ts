import React from 'react';

import { useTextFieldItemContext } from '../TextFieldItemContext';
import { useHandleValueChange } from './use-handle-value-change';

export function useHandleClearButtonClick() {
  const {
    id,
    textField: { inputRef },
  } = useTextFieldItemContext();

  const handleValueChange = useHandleValueChange();

  const handleClearButtonClick = React.useCallback(() => {
    handleValueChange(id, '');
    inputRef.current?.focus();
  }, [handleValueChange, id, inputRef]);

  return handleClearButtonClick;
}
