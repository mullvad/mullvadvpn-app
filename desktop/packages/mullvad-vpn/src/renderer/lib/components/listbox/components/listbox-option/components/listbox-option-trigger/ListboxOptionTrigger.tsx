import React from 'react';
import styled, { css } from 'styled-components';

import { colors } from '../../../../../../foundations';
import { useListItemContext } from '../../../../../list-item/ListItemContext';
import { useListboxContext } from '../../../../';
import { useListboxOptionContext } from '../../';
import { StyledListboxOptionItem } from '../';

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
          ${StyledListboxOptionItem} {
            background-color: ${colors.whiteOnBlue10};
          }
        }

        &&:active {
          ${StyledListboxOptionItem} {
            background-color: ${colors.whiteOnBlue20};
          }
        }
      `;
    }

    return null;
  }}
`;

export const ListboxOptionTrigger = ({ children, ...props }: ListboxOptionTriggerProps) => {
  const { value } = useListboxOptionContext();
  const { disabled } = useListItemContext();
  const triggerRef = React.useRef<HTMLLIElement>(null);

  const { value: selectedValue, onValueChange } = useListboxContext();
  const selected = value === selectedValue;

  const handleTriggerClick = React.useCallback(async () => {
    if (onValueChange) {
      await onValueChange(value);
    }
  }, [onValueChange, value]);

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
      data-option
      role="option"
      aria-selected={selected}
      aria-disabled={disabled}
      tabIndex={-1}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      $disabled={disabled}
      {...props}>
      {children}
    </StyledListItemOptionTrigger>
  );
};
