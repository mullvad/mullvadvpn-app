import React from 'react';

import { useRovingFocus } from '../../../../hooks';
import { useListboxContext } from '../../';

export type ListboxOptionsProps = {
  children: React.ReactNode[];
};

export function ListboxOptions({ children }: ListboxOptionsProps) {
  const { labelId, optionsRef, focusedIndex, setFocusedIndex } = useListboxContext();
  const { handleFocus, handleKeyboardNavigation, handleBlur, tabIndex } = useRovingFocus({
    focusedIndex,
    optionsRef,
    setFocusedIndex,
    selector: '[data-option="true"]:not([aria-disabled="true"])',
  });

  const onKeyDown = React.useCallback(
    (event: React.KeyboardEvent) => {
      handleKeyboardNavigation(event);
    },
    [handleKeyboardNavigation],
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
