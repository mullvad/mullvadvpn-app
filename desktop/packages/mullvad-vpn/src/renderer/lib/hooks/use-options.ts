import React from 'react';

import { getInitialOption, getOptions } from '../utils';
import { useHandleKeyboardNavigation } from './use-handle-keyboard-navigation';

export type UseOptionsProps<T extends HTMLElement> = {
  optionsRef: React.RefObject<T | null>;
  focusedIndex?: number;
  setFocusedIndex: React.Dispatch<React.SetStateAction<number | undefined>>;
};

export function useOptions<T extends HTMLElement>({
  optionsRef,
  focusedIndex,
  setFocusedIndex,
}: UseOptionsProps<T>) {
  const [tabIndex, setTabIndex] = React.useState<number>(0);
  const handleFocus = React.useCallback(
    (event: React.FocusEvent) => {
      if (!optionsRef.current?.isSameNode(event.target)) return;

      const options = getOptions(optionsRef.current);

      const initialOption = getInitialOption(options);
      if (initialOption) {
        setTabIndex(-1);
        initialOption.focus();
      }
    },
    [optionsRef],
  );

  const handleKeyboardNavigation = useHandleKeyboardNavigation({
    optionsRef,
    setFocusedIndex,
    focusedIndex,
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
