import React from 'react';

import { useListboxContext } from '../../../ListboxContext';
import { getOptions } from '../utils';

export const useFocusOptionByIndex = () => {
  const { setFocusedIndex, optionsRef } = useListboxContext();
  return React.useCallback(
    (index: number) => {
      const options = getOptions(optionsRef.current);
      if (options) {
        setFocusedIndex(index);
        const option: HTMLElement = options[index];
        option.focus();
      }
    },
    [optionsRef, setFocusedIndex],
  );
};
