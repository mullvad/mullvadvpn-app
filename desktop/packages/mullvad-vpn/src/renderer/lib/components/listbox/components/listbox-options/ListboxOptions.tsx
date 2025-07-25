import React from 'react';

import { useListboxContext } from '../listbox-context';
import { useChildrenValues, useHandleKeyboardNavigation } from './hooks';

export type ListboxOptionsProps = {
  children: React.ReactNode[];
};

export function ListboxOptions({ children }: ListboxOptionsProps) {
  const { setFocusedValue } = useListboxContext();
  const ref = React.useRef<HTMLDivElement>(null);
  const values = useChildrenValues(children);

  const handleKeyboardNavigation = useHandleKeyboardNavigation(values);
  const onKeyDown = React.useCallback(
    async (event: React.KeyboardEvent) => {
      await handleKeyboardNavigation(event);
    },
    [handleKeyboardNavigation],
  );

  const onBlur = React.useCallback(
    (e: React.FocusEvent<HTMLDivElement>) => {
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
    <div ref={ref} onKeyDown={onKeyDown} onBlur={onBlur}>
      {children}
    </div>
  );
}
