import React from 'react';

import { useListboxContext } from '../../../ListboxContext';
import { getOptions } from '../utils';

export const useFocusOptionByIndex = () => {
  const { setFocusedIndex, optionsRef } = useListboxContext();
  return React.useCallback(
    (index: number) => {
      const options = getOptions(optionsRef.current);
      setFocusedIndex(index);
      const option = options[index];
      option.focus();
    },
    [optionsRef, setFocusedIndex],
  );
};
