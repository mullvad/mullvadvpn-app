import React from 'react';

import { useListboxContext } from '../../listbox-context';

export const useHandleKeyboardNavigation = <T>(options: T[]) => {
  const {
    value: selectedValue,
    focusedValue,
    setFocusedValue,
    onValueChange,
  } = useListboxContext<T>();

  return React.useCallback(
    async (event: React.KeyboardEvent) => {
      if (event.key === 'Space' || event.key === 'Enter') {
        if (onValueChange && focusedValue) {
          await onValueChange(focusedValue);
        }
      }

      // Use roving tabindex to determine the next focusable option
      const value = focusedValue || selectedValue || options[0];
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
    [focusedValue, selectedValue, options, onValueChange, setFocusedValue],
  );
};
