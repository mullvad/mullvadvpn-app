import React from 'react';

import { getOptions } from '../../../utils';

export const useFocusOptionByIndex = <T extends HTMLElement>({
  optionsRef,
  setFocusedIndex,
  selector,
}: {
  optionsRef: React.RefObject<T | null>;
  setFocusedIndex: (index: number) => void;
  selector: string;
}) => {
  return React.useCallback(
    (index: number) => {
      const options = getOptions(optionsRef.current, selector);
      setFocusedIndex(index);
      const option = options[index];
      option.focus();
    },
    [optionsRef, selector, setFocusedIndex],
  );
};
