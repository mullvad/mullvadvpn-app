import React from 'react';

import { getOptions } from '../utils';

export const useFocusOptionByIndex = ({
  optionsRef,
  setFocusedIndex,
}: {
  optionsRef: React.RefObject<HTMLUListElement | null>;
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
