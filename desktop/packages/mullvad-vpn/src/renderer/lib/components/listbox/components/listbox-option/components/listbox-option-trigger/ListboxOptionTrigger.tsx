import React from 'react';
import styled, { css } from 'styled-components';

import { colors } from '../../../../../../foundations';
import { useListItem } from '../../../../../list-item/ListItemContext';
import { useListboxContext } from '../../../listbox-context';
import { useListboxOptionContext } from '../';
import { StyledListItemOptionItem } from '../';

export type ListboxOptionTriggerProps = React.ComponentPropsWithRef<'li'>;

export const StyledListItemOptionTrigger = styled.li<{ $disabled?: boolean }>`
  display: flex;
  width: 100%;
  background-color: transparent;

  &&:focus-visible {
    outline: 2px solid ${colors.white};
    outline-offset: -2px;
    z-index: 10;
  }

  ${({ $disabled }) => {
    if (!$disabled) {
      return css`
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
    }

    return null;
  }}
`;

export const ListboxOptionTrigger = ({ children, ...props }: ListboxOptionTriggerProps) => {
  const { value } = useListboxOptionContext();
  const { disabled } = useListItem();
  const triggerRef = React.useRef<HTMLLIElement>(null);

  const {
    value: selectedValue,
    onValueChange,
    focusedValue,
    setFocusedValue,
  } = useListboxContext();
  const selected = value === selectedValue;
  const focused = value === focusedValue;

  const tabIndex = !disabled && selected && focusedValue === undefined ? 0 : -1;

  const handleTriggerClick = React.useCallback(async () => {
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

  const handleClick = !disabled ? handleTriggerClick : undefined;

  const handleKeyDown = React.useCallback(
    async (event: React.KeyboardEvent) => {
      if (disabled) return;
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        if (onValueChange) {
          await onValueChange(value);
        }
      }
    },
    [disabled, onValueChange, value],
  );

  return (
    <StyledListItemOptionTrigger
      ref={triggerRef}
      role="option"
      aria-selected={selected}
      aria-disabled={disabled}
      tabIndex={tabIndex}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      onFocus={onFocus}
      $disabled={disabled}
      {...props}>
      {children}
    </StyledListItemOptionTrigger>
  );
};
