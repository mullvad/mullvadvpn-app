import React from 'react';

import { useListboxContext } from '../../../';
import { getOptions } from '../utils';
import { useFocusOptionByIndex } from './useFocusOptionByIndex';
import { useGetInitialFocusIndex } from './useGetInitialFocusIndex';

export const useHandleKeyboardNavigation = () => {
  const { optionsRef } = useListboxContext();
  const getInitialFocusIndex = useGetInitialFocusIndex();
  const focusOptionByIndex = useFocusOptionByIndex();

  return React.useCallback(
    (event: React.KeyboardEvent) => {
      const options = getOptions(optionsRef.current);
      if (!options) return;

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
