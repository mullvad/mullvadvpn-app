import React from 'react';

import { getOptions } from '../utils';
import { useFocusOptionByIndex } from './use-focus-option-by-index';
import { useGetInitialFocusIndex } from './use-get-initial-focus-index';

export const useHandleKeyboardNavigation = <T extends HTMLElement>({
  optionsRef,
  focusedIndex,
  setFocusedIndex,
}: {
  optionsRef: React.RefObject<T | null>;
  focusedIndex?: number;
  setFocusedIndex: (index: number) => void;
}) => {
  const getInitialFocusIndex = useGetInitialFocusIndex({ optionsRef, focusedIndex });
  const focusOptionByIndex = useFocusOptionByIndex({
    optionsRef,
    setFocusedIndex,
  });

  return React.useCallback(
    (event: React.KeyboardEvent) => {
      const options = getOptions(optionsRef.current);

      const initialFocusedIndex = getInitialFocusIndex();

      if (event.key === 'ArrowUp') {
        event.preventDefault();
        if (initialFocusedIndex > 0) {
          const newFocusedIndex = initialFocusedIndex - 1;
          focusOptionByIndex(newFocusedIndex);
        }
      } else if (event.key === 'ArrowDown') {
        event.preventDefault();
        if (initialFocusedIndex < options.length - 1) {
          const newFocusedIndex = initialFocusedIndex + 1;
          focusOptionByIndex(newFocusedIndex);
        }
      } else if (event.key === 'Home') {
        event.preventDefault();
        focusOptionByIndex(0);
      } else if (event.key === 'End') {
        event.preventDefault();
        focusOptionByIndex(options.length - 1);
      }
    },
    [focusOptionByIndex, getInitialFocusIndex, optionsRef],
  );
};
