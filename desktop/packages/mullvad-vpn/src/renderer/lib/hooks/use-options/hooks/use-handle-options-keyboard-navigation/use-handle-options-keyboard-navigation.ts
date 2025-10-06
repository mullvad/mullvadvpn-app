import React from 'react';

import { getOptions } from '../../utils';
import { useFocusOptionByIndex, useGetInitialFocusIndex } from './hooks';

export type KeyboardNavigationOrientation = 'horizontal' | 'vertical';

export const useHandleOptionsKeyboardNavigation = <T extends HTMLElement>({
  optionsRef,
  focusedIndex,
  setFocusedIndex,
  orientation = 'vertical',
  selector,
}: {
  optionsRef: React.RefObject<T | null>;
  focusedIndex?: number;
  setFocusedIndex: (index: number) => void;
  orientation?: KeyboardNavigationOrientation;
  selector: string;
}) => {
  const getInitialFocusIndex = useGetInitialFocusIndex({ optionsRef, focusedIndex, selector });
  const focusOptionByIndex = useFocusOptionByIndex({
    optionsRef,
    setFocusedIndex,
    selector,
  });

  const nextKey = orientation === 'vertical' ? 'ArrowDown' : 'ArrowRight';
  const previousKey = orientation === 'vertical' ? 'ArrowUp' : 'ArrowLeft';

  return React.useCallback(
    (event: React.KeyboardEvent) => {
      const options = getOptions(optionsRef.current, selector);

      const initialFocusedIndex = getInitialFocusIndex();

      if (event.key === previousKey) {
        event.preventDefault();
        if (initialFocusedIndex > 0) {
          const newFocusedIndex = initialFocusedIndex - 1;
          focusOptionByIndex(newFocusedIndex);
        }
      } else if (event.key === nextKey) {
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
    [focusOptionByIndex, getInitialFocusIndex, nextKey, optionsRef, previousKey, selector],
  );
};
