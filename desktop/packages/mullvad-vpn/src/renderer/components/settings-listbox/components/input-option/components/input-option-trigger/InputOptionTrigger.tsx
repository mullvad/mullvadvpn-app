import React from 'react';
import styled from 'styled-components';

import { useListboxContext } from '../../../../../../lib/components/listbox/components';
import {
  ListboxOptionTriggerProps,
  StyledListItemOptionItem,
  useListboxOptionContext,
} from '../../../../../../lib/components/listbox/components/listbox-option/components';
import { colors } from '../../../../../../lib/foundations';
import { useInputOptionContext } from '../../InputOptionContext';

export type InputOptionTriggerProps = ListboxOptionTriggerProps;

export const StyledInputOptionTrigger = styled.li`
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

export const InputOptionTrigger = ({ children, ...props }: InputOptionTriggerProps) => {
  const { value } = useListboxOptionContext();
  const { inputRef } = useInputOptionContext();

  const { value: selectedValue, focusedValue, setFocusedValue } = useListboxContext();
  const selected = value === selectedValue;
  const focused = value === focusedValue;

  const tabIndex = selected && focusedValue === undefined ? 0 : -1;

  React.useEffect(() => {
    if (focused && inputRef.current) {
      inputRef.current.focus();
    }
  }, [value, focused, inputRef]);

  const handleFocus = React.useCallback(() => {
    if (!focused) {
      setFocusedValue(value);
      inputRef.current?.focus();
    }
  }, [focused, inputRef, setFocusedValue, value]);

  return (
    <StyledInputOptionTrigger
      role="option"
      aria-selected={selected}
      tabIndex={tabIndex}
      onFocus={handleFocus}
      {...props}>
      {children}
    </StyledInputOptionTrigger>
  );
};
