import React from 'react';

import { type KeyboardNavigationOrientation, useHandleOptionsKeyboardNavigation } from './hooks';
import { getInitialOption, getOptions } from './utils';

export type UseRovingFocusProps<T extends HTMLElement> = {
  optionsRef: React.RefObject<T | null>;
  focusedIndex?: number;
  setFocusedIndex: React.Dispatch<React.SetStateAction<number | undefined>>;
  selector: string;
  orientation?: KeyboardNavigationOrientation;
};

export function useRovingFocus<T extends HTMLElement>({
  optionsRef,
  focusedIndex,
  setFocusedIndex,
  selector,
  orientation = 'vertical',
}: UseRovingFocusProps<T>) {
  const [tabIndex, setTabIndex] = React.useState<number>(0);
  const handleFocus = React.useCallback(
    (event: React.FocusEvent) => {
      if (!optionsRef.current?.isSameNode(event.target)) return;

      const options = getOptions(optionsRef.current, selector);

      const initialOption = getInitialOption(options);
      if (initialOption) {
        // Prevent the container from being tabbable once an option has focus
        setTabIndex(-1);
        initialOption.focus();
      }
    },
    [optionsRef, selector],
  );

  const handleKeyboardNavigation = useHandleOptionsKeyboardNavigation({
    optionsRef,
    setFocusedIndex,
    focusedIndex,
    selector,
    orientation,
  });

  const handleBlur = React.useCallback(
    (event: React.FocusEvent<T>) => {
      const container = optionsRef.current;
      const nextFocus = event.relatedTarget as Node | null;

      // If focus moves outside the container
      if (!container || !nextFocus || !container.contains(nextFocus)) {
        setFocusedIndex(undefined);
        setTabIndex(0);
      }
    },
    [optionsRef, setFocusedIndex],
  );

  return { tabIndex, handleFocus, handleKeyboardNavigation, handleBlur, optionsRef };
}
