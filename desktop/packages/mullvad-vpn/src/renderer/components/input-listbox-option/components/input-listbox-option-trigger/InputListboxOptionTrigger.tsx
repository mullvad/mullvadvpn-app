import React from 'react';
import styled from 'styled-components';

import { useListboxContext } from '../../../../lib/components/listbox/components';
import { ListboxOptionTriggerProps } from '../../../../lib/components/listbox/components/listbox-option/components';
import { useListboxOptionContext } from '../../../../lib/components/listbox/components/listbox-option/components/listbox-option-context/ListboxOptionContext';
import { StyledListItemOptionItem } from '../../../../lib/components/listbox/components/listbox-option/components/listbox-option-item/ListboxOptionItem';
import { colors } from '../../../../lib/foundations';
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
    <StyledListItemOptionTrigger
      role="option"
      aria-selected={selected}
      tabIndex={tabIndex}
      onFocus={handleFocus}
      {...props}>
      {children}
    </StyledListItemOptionTrigger>
  );
};
