import React from 'react';
import styled, { css, RuleSet } from 'styled-components';

import { Checkbox } from '../checkbox';
import {
  ListItemActionGroup,
  ListItemFooter,
  ListItemFooterText,
  ListItemGroup,
  ListItemIcon,
  ListItemItem,
  ListItemLabel,
  ListItemText,
  ListItemTextField,
  ListItemTrailingAction,
  ListItemTrailingActions,
  ListItemTrigger,
  StyledListItemItem,
  StyledListItemTrailingAction,
  StyledListItemTrailingActions,
  StyledListItemTrigger,
} from './components';
import { useListItemAnimation, useMaxLevel } from './hooks';
import { ListItemProvider } from './ListItemContext';

export type ListItemAnimation = 'flash' | 'dim';
export type ListItemPositions = 'first' | 'middle' | 'last' | 'solo' | 'auto';

export const StyledListItemRoot = styled.div``;

export const StyledListItem = styled(StyledListItemRoot)<{
  $position?: ListItemPositions;
  $animation?: RuleSet<object>;
}>`
  ${({ $animation, $position }) => {
    return css`
      --disabled-border-radius: 0;

      display: grid;
      grid-template-columns: 1fr;

      // If it has a trailing action at the end
      &&:has(> ${StyledListItemTrailingActions}, > ${StyledListItemTrigger}:nth-child(2)) {
        grid-template-columns: 1fr auto;
        ${StyledListItemItem} {
          border-top-right-radius: var(--disabled-border-radius);
          border-bottom-right-radius: var(--disabled-border-radius);
        }
      }

      // If position is auto or last, make top border radius dynamic
      ${() => {
        if ($position === 'auto' || $position === 'last') {
          return css`
            // If directly preceded by another ListItem
            ${StyledListItemRoot} + & {
              margin-top: 1px;
              ${StyledListItemItem} {
                border-top-left-radius: var(--disabled-border-radius);
                border-top-right-radius: var(--disabled-border-radius);
              }
              ${StyledListItemTrailingAction} {
                border-top-right-radius: var(--disabled-border-radius);
              }
            }
          `;
        }

        return null;
      }}

      // If position is auto or first, make bottom border radius dynamic
      ${() => {
        if ($position === 'auto' || $position === 'first') {
          return css`
            // If directly followed by another ListItem
            &:has(+ ${StyledListItemRoot}) {
              ${StyledListItemItem} {
                border-bottom-left-radius: var(--disabled-border-radius);
                border-bottom-right-radius: var(--disabled-border-radius);
              }
              ${StyledListItemTrailingAction} {
                border-bottom-right-radius: var(--disabled-border-radius);
              }
            }
          `;
        }

        return null;
      }}

      // If position is middle or last, remove top border radius
      ${() => {
        if ($position === 'middle' || $position === 'last') {
          return css`
            && {
              margin-top: 1px;
              ${StyledListItemItem} {
                border-top-left-radius: var(--disabled-border-radius);
                border-top-right-radius: var(--disabled-border-radius);
              }
              ${StyledListItemTrailingAction} {
                border-top-right-radius: var(--disabled-border-radius);
              }
            }
          `;
        }

        return null;
      }}

      // If position is middle or first, remove bottom border radius
      ${() => {
        if ($position === 'middle') {
          return css`
            && {
              ${StyledListItemItem} {
                border-bottom-left-radius: var(--disabled-border-radius);
                border-bottom-right-radius: var(--disabled-border-radius);
              }
              ${StyledListItemTrailingAction} {
                border-bottom-right-radius: var(--disabled-border-radius);
              }
            }
          `;
        }

        return null;
      }}

      ${$animation}
    `;
  }}
`;

export type ListItemProps = {
  level?: number;
  position?: ListItemPositions;
  disabled?: boolean;
  animation?: ListItemAnimation | false;
  children: React.ReactNode;
} & React.ComponentPropsWithRef<'div'>;

const ListItem = ({
  level: levelProp = 0,
  position = 'auto',
  disabled,
  animation: animationProp,
  children,
  ...props
}: ListItemProps) => {
  const animation = useListItemAnimation(animationProp);
  const level = useMaxLevel(levelProp);
  return (
    <ListItemProvider
      level={level}
      position={position}
      disabled={disabled}
      animation={animationProp}>
      <StyledListItem
        $position={position}
        $animation={animationProp == 'dim' ? animation : undefined}
        {...props}>
        {children}
      </StyledListItem>
    </ListItemProvider>
  );
};

const ListItemNamespace = Object.assign(ListItem, {
  Label: ListItemLabel,
  Group: ListItemGroup,
  ActionGroup: ListItemActionGroup,
  Text: ListItemText,
  Trigger: ListItemTrigger,
  Item: ListItemItem,
  Footer: ListItemFooter,
  FooterText: ListItemFooterText,
  Icon: ListItemIcon,
  TextField: ListItemTextField,
  Checkbox: Checkbox,
  TrailingActions: ListItemTrailingActions,
  TrailingAction: ListItemTrailingAction,
});

export { ListItemNamespace as ListItem };
