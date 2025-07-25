import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../../../../foundations';
import { ListItem } from '../../../../../list-item';
import { ListItemTriggerProps } from '../../../../../list-item/components';
import { useListboxContext } from '../../../listbox-context';
import { useListboxOptionContext } from '../listbox-option-context/ListboxOptionContext';
import { StyledListItemOptionItem } from '../listbox-option-item/ListboxOptionItem';

export type ListboxOptionTriggerProps = ListItemTriggerProps;

export const StyledListItemOptionTrigger = styled(ListItem.Trigger)`
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

export const ListboxOptionTrigger = ({ children, ...props }: ListboxOptionTriggerProps) => {
  const triggerRef = React.useRef<HTMLButtonElement>(null);
  const { value } = useListboxOptionContext();

  const {
    value: selectedValue,
    onValueChange,
    focusedValue,
    setFocusedValue,
  } = useListboxContext();
  const selected = value === selectedValue;
  const focused = value === focusedValue;

  const onTriggerClick = React.useCallback(async () => {
    if (onValueChange) {
      await onValueChange(value);
    }
  }, [onValueChange, value]);

  React.useEffect(() => {
    if (focused && triggerRef.current) {
      triggerRef.current.focus();
    }
  }, [value, focused]);

  const onFocus = React.useCallback(() => {
    if (focused) return;
    setFocusedValue(value);
  }, [focused, setFocusedValue, value]);

  // TODO: can focus logic be cleaned up?
  return (
    <StyledListItemOptionTrigger
      ref={triggerRef}
      role="option"
      aria-selected={selected}
      tabIndex={selected && focusedValue === undefined ? 0 : -1}
      onClick={onTriggerClick}
      onFocus={onFocus}
      {...props}>
      {children}
    </StyledListItemOptionTrigger>
  );
};
