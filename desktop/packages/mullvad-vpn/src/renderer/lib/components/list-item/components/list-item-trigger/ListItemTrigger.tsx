import React from 'react';
import styled, { css } from 'styled-components';

import { colors } from '../../../../foundations';
import { useListItemContext } from '../../ListItemContext';
import { StyledListItemItem } from '../list-item-item';
import { StyledListItemTrailingAction } from '../list-item-trailing-action';

export const StyledListItemTrigger = styled.div<{ $disabled?: boolean }>`
  display: flex;
  background-color: transparent;

  &&:focus-visible {
    outline: 2px solid ${colors.white};
    outline-offset: -2px;
    z-index: 10;
  }

  ${({ $disabled }) => {
    if (!$disabled) {
      return css`
        &:hover {
          ${StyledListItemItem} {
            background-color: ${colors.whiteOnBlue10};
          }
          ${StyledListItemTrailingAction} {
            background-color: ${colors.whiteOnBlue10};
          }
        }

        &:active {
          ${StyledListItemItem} {
            background-color: ${colors.whiteOnBlue20};
          }
          ${StyledListItemTrailingAction} {
            background-color: ${colors.whiteOnBlue20};
          }
        }
      `;
    }

    return null;
  }}
`;

export type ListItemTriggerProps = React.ComponentPropsWithRef<'div'>;

export function ListItemTrigger({ onClick, ...props }: ListItemTriggerProps) {
  const { disabled } = useListItemContext();

  const handleClick = React.useCallback(
    (event: React.MouseEvent<HTMLDivElement>) => {
      if (disabled) {
        return;
      }
      onClick?.(event);
    },
    [disabled, onClick],
  );

  const handleKeyDown = React.useCallback(
    (event: React.KeyboardEvent<HTMLDivElement>) => {
      if (disabled) {
        return;
      }
      if (event.key === 'Enter' || event.key === ' ') {
        if (event.key === ' ') {
          event.preventDefault();
        }

        event.currentTarget.click();
      }
    },
    [disabled],
  );

  return (
    <StyledListItemTrigger
      role="button"
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      tabIndex={disabled ? -1 : 0}
      aria-disabled={disabled ? true : undefined}
      $disabled={disabled}
      {...props}
    />
  );
}
