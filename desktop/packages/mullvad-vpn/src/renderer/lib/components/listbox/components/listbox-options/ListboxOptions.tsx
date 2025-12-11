import React from 'react';
import styled from 'styled-components';

import { useRovingFocus } from '../../../../hooks';
import { StyledListItem } from '../../../list-item';
import { StyledListItemItem } from '../../../list-item/components';
import { useListboxContext } from '../../';

export type ListboxOptionsProps = {
  children: React.ReactNode[];
};

export const StyledListboxOptions = styled.ul`
  display: flex;
  flex-direction: column;
  gap: 1px;

  // If the option is follow by another list item, remove bottom border radius
  && > ${StyledListItem}:has(+ ${StyledListItem}) ${StyledListItemItem} {
    border-bottom-left-radius: 0;
    border-bottom-right-radius: 0;
  }

  // If the option is preceded by another list item, remove top border radius
  && > ${StyledListItem} + ${StyledListItem} ${StyledListItemItem} {
    border-top-left-radius: 0;
    border-top-right-radius: 0;
  }
`;

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
