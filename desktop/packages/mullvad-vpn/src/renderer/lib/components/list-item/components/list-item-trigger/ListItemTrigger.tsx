import React from 'react';
import styled, { css } from 'styled-components';

import { colors } from '../../../../foundations';
import { useListItemContext } from '../../ListItemContext';
import { StyledListItemItem } from '../list-item-item';
import { StyledListItemTrailingAction } from '../list-item-trailing-action';
import { StyledListItemTrailingActions } from '../list-item-trailing-actions';
import { ListItemTriggerProvider } from './ListItemTriggerContext';

export const StyledListItemTrigger = styled.button<{ disabled?: boolean }>`
  display: flex;
  background-color: transparent;

  &&:focus-visible {
    outline: 2px solid ${colors.white};
    outline-offset: -2px;
    z-index: 10;
  }

  ${({ disabled }) => {
    if (!disabled) {
      return css`
        &:hover {
          ${StyledListItemItem} {
            background-color: ${colors.whiteOnBlue10};
          }
          ${StyledListItemTrailingAction} {
            background-color: ${colors.whiteOnBlue10};
          }
          ~ ${StyledListItemTrailingActions} {
            & > ${StyledListItemTrailingAction}:not(:last-child) {
              background-color: ${colors.whiteOnBlue10};
            }
          }
        }

        &:active {
          ${StyledListItemItem} {
            background-color: ${colors.whiteOnBlue20};
          }
          ${StyledListItemTrailingAction} {
            background-color: ${colors.whiteOnBlue20};
          }
          ~ ${StyledListItemTrailingActions} {
            & > ${StyledListItemTrailingAction}:not(:last-child) {
              background-color: ${colors.whiteOnBlue20};
            }
          }
        }
      `;
    }

    return null;
  }}
`;

export type ListItemTriggerProps = React.ComponentPropsWithRef<'button'> & {
  disabled?: boolean;
};

export function ListItemTrigger({
  onClick,
  disabled: disabledProp,
  ...props
}: ListItemTriggerProps) {
  const { disabled: disabledContext } = useListItemContext();
  const disabled = disabledProp ?? disabledContext;

  return (
    <ListItemTriggerProvider disabled={disabled}>
      <StyledListItemTrigger
        onClick={onClick}
        disabled={disabled}
        tabIndex={disabled ? -1 : 0}
        aria-disabled={disabled ? true : undefined}
        {...props}
      />
    </ListItemTriggerProvider>
  );
}
