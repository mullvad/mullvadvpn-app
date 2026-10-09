import React from 'react';

import { useSelectLocationViewContext } from '../../../../../SelectLocationViewContext';
import { useTextFieldItemContext } from '../TextFieldItemContext';
import { useHandleValueChange } from './use-handle-value-change';

export function useHandleClearButtonClick() {
  const {
    id,
    textField: { inputRef },
  } = useTextFieldItemContext();
  const { setIsolatedItem } = useSelectLocationViewContext();

  const handleValueChange = useHandleValueChange();

  const handleClearButtonClick = React.useCallback(() => {
    handleValueChange(id, '');
    setIsolatedItem(undefined);
    inputRef.current?.focus();
  }, [handleValueChange, id, inputRef, setIsolatedItem]);

  return handleClearButtonClick;
}
