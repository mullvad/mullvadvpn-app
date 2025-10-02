import React from 'react';

import { getOptions } from '../utils';

export const useFocusOptionByIndex = <T extends HTMLElement>({
  optionsRef,
  setFocusedIndex,
}: {
  optionsRef: React.RefObject<T | null>;
  setFocusedIndex: (index: number) => void;
}) => {
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
