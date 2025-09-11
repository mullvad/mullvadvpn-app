import React from 'react';
import styled from 'styled-components';

import { useListboxContext } from '../../../../../../lib/components/listbox/components';
import {
  ListboxOptionTriggerProps,
  StyledListItemOptionItem,
  useListboxOptionContext,
} from '../../../../../../lib/components/listbox/components/listbox-option/components';
import { colors } from '../../../../../../lib/foundations';
import { useInputListboxOption } from '../input-listbox-option-context';

export type InputListboxOptionTriggerProps = ListboxOptionTriggerProps;

export const StyledListItemOptionTrigger = styled.li`
  &&[aria-selected='true'] {
    &:hover {
      ${StyledListItemOptionItem} {
        background-color: ${colors.green};
      }
    }
    &:active {
      ${StyledListItemOptionItem} {
        background-color: ${colors.green};
      }
    }
  }
`;

export const InputListboxOptionTrigger = ({
  children,
  ...props
}: InputListboxOptionTriggerProps) => {
  const { value } = useListboxOptionContext();
  const { inputRef } = useInputListboxOption();

  const { value: selectedValue, focusedValue, setFocusedValue } = useListboxContext();
  const selected = value === selectedValue;
  const focused = value === focusedValue;

  React.useEffect(() => {
    if (focused && inputRef.current) {
      inputRef.current.focus();
    }
  }, [value, focused, inputRef]);

  const handleFocus = React.useCallback(() => {
    if (focused) return;
    setFocusedValue(value);
    inputRef.current?.focus();
  }, [focused, inputRef, setFocusedValue, value]);

  return (
    <StyledListItemOptionTrigger
      role="option"
      aria-selected={selected}
      tabIndex={selected && focusedValue === undefined ? 0 : -1}
      onFocus={handleFocus}
      {...props}>
      {children}
    </StyledListItemOptionTrigger>
  );
};
