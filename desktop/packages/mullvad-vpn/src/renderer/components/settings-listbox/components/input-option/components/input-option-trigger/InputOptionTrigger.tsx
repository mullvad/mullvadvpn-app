import React from 'react';
import styled from 'styled-components';

import { useListboxContext } from '../../../../../../lib/components/listbox/';
import { useListboxOptionContext } from '../../../../../../lib/components/listbox/components/listbox-option/';
import {
  ListboxOptionTriggerProps,
  StyledListItemOptionItem,
} from '../../../../../../lib/components/listbox/components/listbox-option/components';
import { colors } from '../../../../../../lib/foundations';
import { useInputOptionContext } from '../../InputOptionContext';

export type InputOptionTriggerProps = ListboxOptionTriggerProps;

export const StyledInputOptionTrigger = styled.li`
  &&:hover {
    ${StyledListItemOptionItem} {
      background-color: ${colors.whiteOnBlue10};
    }
  }

  &&:active {
    ${StyledListItemOptionItem} {
      background-color: ${colors.whiteOnBlue20};
    }
  }

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
  const {
    inputRef,
    inputState: { value: inputValue },
  } = useInputOptionContext();

  const {
    value: selectedValue,
    focusedValue,
    setFocusedValue,
    onValueChange,
  } = useListboxContext();
  const selected = value === selectedValue;
  const focused = value === focusedValue;

  const tabIndex = selected && focusedValue === undefined ? 0 : -1;

  React.useEffect(() => {
    if (focused && inputRef.current) {
      inputRef.current.focus();
    }
  }, [value, focused, inputRef]);

  const handleClick = React.useCallback(async () => {
    setFocusedValue(value);
    inputRef.current?.focus();
    if (!selected) {
      await onValueChange?.(inputValue);
    }
  }, [inputRef, inputValue, onValueChange, selected, setFocusedValue, value]);

  const handleFocus = React.useCallback(() => {
    setFocusedValue(value);
    inputRef.current?.focus();
  }, [inputRef, setFocusedValue, value]);

  return (
    <StyledInputOptionTrigger
      role="option"
      aria-selected={selected}
      tabIndex={tabIndex}
      onFocus={handleFocus}
      onClick={handleClick}
      {...props}>
      {children}
    </StyledInputOptionTrigger>
  );
};
