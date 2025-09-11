import React from 'react';

import { useListboxContext } from '../../';
import { useChildrenValues, useHandleKeyboardNavigation } from './hooks';

export type ListboxOptionsProps = {
  children: React.ReactNode[];
};

export function ListboxOptions({ children }: ListboxOptionsProps) {
  const { labelId, setFocusedValue } = useListboxContext();
  const ref = React.useRef<HTMLUListElement>(null);
  const values = useChildrenValues(children);

  const handleKeyboardNavigation = useHandleKeyboardNavigation(values);
  const onKeyDown = React.useCallback(
    (event: React.KeyboardEvent) => {
      handleKeyboardNavigation(event);
    },
    [handleKeyboardNavigation],
  );

  const onBlur = React.useCallback(
    (e: React.FocusEvent<HTMLUListElement>) => {
      const container = ref.current;
      const nextFocus = e.relatedTarget as Node | null;

      // If focus moves outside the listbox
      if (!container || !nextFocus || !container.contains(nextFocus)) {
        setFocusedValue(undefined);
      }
    },
    [setFocusedValue],
  );

  return (
    <ul ref={ref} role="listbox" onKeyDown={onKeyDown} onBlur={onBlur} aria-labelledby={labelId}>
      {children}
    </ul>
  );
}
