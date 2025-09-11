import React from 'react';

import { useListboxContext } from '../../../';

export const useHandleKeyboardNavigation = <T>(options: T[]) => {
  const { value: selectedValue, focusedValue, setFocusedValue } = useListboxContext<T>();

  return React.useCallback(
    (event: React.KeyboardEvent) => {
      let value: T;
      if (focusedValue !== undefined) value = focusedValue;
      else if (selectedValue !== undefined) value = selectedValue;
      else value = options[0];

      // Use roving tabindex to determine the next focusable option
      let nextValue: T | undefined;
      if (event.key === 'ArrowUp') {
        event.preventDefault();
        const focusedOptionIndex = options.findIndex((option) => option === value);
        if (focusedOptionIndex <= 0) return;
        const nextOptionIndex = focusedOptionIndex - 1;
        nextValue = options[nextOptionIndex];
      } else if (event.key === 'ArrowDown') {
        event.preventDefault();
        const focusedOptionIndex = options.findIndex((option) => option === value);
        if (focusedOptionIndex >= options.length - 1) return;
        const nextOptionIndex = focusedOptionIndex + 1;
        if (nextOptionIndex === -1) return;
        nextValue = options[nextOptionIndex];
      } else if (event.key === 'Home') {
        event.preventDefault();
        nextValue = options[0];
      } else if (event.key === 'End') {
        event.preventDefault();
        nextValue = options[options.length - 1];
      }
      if (nextValue !== undefined) {
        setFocusedValue(nextValue);
      }
    },
    [focusedValue, selectedValue, options, setFocusedValue],
  );
};
