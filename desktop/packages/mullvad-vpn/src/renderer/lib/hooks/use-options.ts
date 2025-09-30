import React from 'react';

import { getInitialOption, getOptions } from '../utils';
import { useHandleKeyboardNavigation } from './use-handle-keyboard-navigation';

export type UseOptionsProps = {
  optionsRef: React.RefObject<HTMLUListElement | null>;
  focusedIndex?: number;
  setFocusedIndex: React.Dispatch<React.SetStateAction<number | undefined>>;
};

export type UseOptionsFieldState = {
  tabIndex: number;
  handleFocus: (event: React.FocusEvent) => void;
  optionsRef: React.RefObject<HTMLUListElement | null>;
  handleKeyboardNavigation: (event: React.KeyboardEvent) => void;
  handleBlur: (event: React.FocusEvent<HTMLUListElement>) => void;
};

export function useOptions({ optionsRef, focusedIndex, setFocusedIndex }: UseOptionsProps) {
  const [tabIndex, setTabIndex] = React.useState<number>(0);
  const handleFocus = React.useCallback(
    (event: React.FocusEvent) => {
      if (!optionsRef.current?.isSameNode(event.target)) return;

      const options = getOptions(optionsRef.current);

      const initialOption = getInitialOption(options);
      if (initialOption) {
        // Prevent the container from being tabbable once an option has focus
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
    (event: React.FocusEvent<HTMLUListElement>) => {
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
