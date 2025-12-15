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
    triggerRef,
    inputState: { value: inputValue, invalid },
  } = useInputOptionContext();

  const { value: selectedValue, onValueChange } = useListboxContext();
  const selected = value === selectedValue;

  const handleClick = React.useCallback(async () => {
    inputRef.current?.focus();
    if (!selected && !invalid) {
      await onValueChange?.(inputValue);
    }
  }, [inputRef, inputValue, invalid, onValueChange, selected]);

  const handleFocus = React.useCallback(() => {
    inputRef.current?.focus();
  }, [inputRef]);

  return (
    <StyledInputOptionTrigger
      ref={triggerRef}
      role="option"
      data-option
      aria-selected={selected}
      tabIndex={-1}
      onFocus={handleFocus}
      onClick={handleClick}
      {...props}>
      {children}
    </StyledInputOptionTrigger>
  );
};
