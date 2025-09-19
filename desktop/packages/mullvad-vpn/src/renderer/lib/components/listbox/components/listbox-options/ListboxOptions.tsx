import React from 'react';

import { useListboxContext } from '../../';
import { useHandleKeyboardNavigation } from './hooks';
import { getInitialOption, getOptions } from './utils';

export type ListboxOptionsProps = {
  children: React.ReactNode[];
};

export function ListboxOptions({ children }: ListboxOptionsProps) {
  const { labelId, optionsRef, setFocusedIndex } = useListboxContext();
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

  const handleKeyboardNavigation = useHandleKeyboardNavigation();

  const onKeyDown = React.useCallback(
    (event: React.KeyboardEvent) => {
      handleKeyboardNavigation(event);
    },
    [handleKeyboardNavigation],
  );

  const handleBlur = React.useCallback(
    (event: React.FocusEvent<HTMLUListElement>) => {
      const container = optionsRef.current;
      const nextFocus = event.relatedTarget as Node | null;

      // If focus moves outside the listbox
      if (!container || !nextFocus || !container.contains(nextFocus)) {
        setFocusedIndex(undefined);
        setTabIndex(0);
      }
    },
    [optionsRef, setFocusedIndex],
  );

  return (
    <ul
      ref={optionsRef}
      role="listbox"
      aria-labelledby={labelId}
      onKeyDown={onKeyDown}
      onBlur={handleBlur}
      onFocus={handleFocus}
      tabIndex={tabIndex}>
      {children}
    </ul>
  );
}
