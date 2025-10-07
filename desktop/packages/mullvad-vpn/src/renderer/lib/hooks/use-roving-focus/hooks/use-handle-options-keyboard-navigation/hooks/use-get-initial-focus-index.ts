import React from 'react';

import { getOptions, getSelectedOptionIndex } from '../../../utils';

export const useGetInitialFocusIndex = <T extends HTMLElement>({
  focusedIndex,
  optionsRef,
  selector,
}: {
  focusedIndex?: number;
  optionsRef: React.RefObject<T | null>;
  selector: string;
}) => {
  return React.useCallback(() => {
    const options = getOptions(optionsRef.current, selector);
    if (focusedIndex !== undefined) {
      return focusedIndex;
    }
    const selectedOptionIndex = getSelectedOptionIndex(options);
    if (selectedOptionIndex !== -1) {
      return selectedOptionIndex;
    }
    return 0;
  }, [focusedIndex, optionsRef, selector]);
};
