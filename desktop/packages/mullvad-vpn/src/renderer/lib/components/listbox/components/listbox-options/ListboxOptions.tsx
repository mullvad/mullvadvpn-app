import React from 'react';
import styled from 'styled-components';

import { useRovingFocus } from '../../../../hooks';
import { useListboxContext } from '../../';
import { ListboxOption } from './components';

export type ListboxOptionsProps = {
  children: React.ReactNode[];
};

export const StyledListboxOptions = styled.ul`
  display: flex;
  flex-direction: column;
`;

function ListboxOptions({ children }: ListboxOptionsProps) {
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
    <StyledListboxOptions
      ref={optionsRef}
      role="listbox"
      aria-labelledby={labelId}
      onKeyDown={onKeyDown}
      onBlur={handleBlur}
      onFocus={handleFocus}
      tabIndex={tabIndex}>
      {children}
    </StyledListboxOptions>
  );
}

const ListboxOptionsNamespace = Object.assign(ListboxOptions, {
  Option: ListboxOption,
});

export { ListboxOptionsNamespace as ListboxOptions };
