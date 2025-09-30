import React from 'react';

import { getOptions, getSelectedOptionIndex } from '../utils';

export const useGetInitialFocusIndex = ({
  focusedIndex,
  optionsRef,
}: {
  focusedIndex?: number;
  optionsRef: React.RefObject<HTMLUListElement | null>;
}) => {
  return React.useCallback(() => {
    const options = getOptions(optionsRef.current);
    if (focusedIndex !== undefined) {
      return focusedIndex;
    }
    const selectedOptionIndex = getSelectedOptionIndex(options);
    if (selectedOptionIndex !== -1) {
      return selectedOptionIndex;
    }
    return 0;
  }, [focusedIndex, optionsRef]);
};
