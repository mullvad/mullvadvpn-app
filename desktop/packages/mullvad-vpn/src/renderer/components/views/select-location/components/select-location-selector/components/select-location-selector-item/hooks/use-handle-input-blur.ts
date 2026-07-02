import React from 'react';

import { useSelectLocationSelectorItemContext } from '../SelectLocationSelectorItemContext';

export function useHandleInputBlur() {
  const {
    textField: { value, reset },
  } = useSelectLocationSelectorItemContext();

  return React.useCallback(() => {
    if (value === '') {
      reset();
    }
  }, [value, reset]);
}
