import React from 'react';
import styled, { css, RuleSet } from 'styled-components';

import {
  ListItemContent,
  ListItemFooter,
  ListItemGroup,
  ListItemIcon,
  ListItemItem,
  ListItemLabel,
  ListItemText,
  ListItemTextField,
  ListItemTrigger,
  StyledListItemItem,
} from './components';
import { useListItemAnimation } from './hooks';
import { levels } from './levels';
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

      ${() => {
        if ($position === 'auto') {
          return css`
            // If directly preceded by another ListItem
            ${StyledListItemRoot} + & {
              ${StyledListItemItem} {
                border-top-left-radius: var(--disabled-border-radius);
                border-top-right-radius: var(--disabled-border-radius);
              }
            }

            // If directly followed by another ListItem
            &:has(+ ${StyledListItemRoot}) {
              margin-bottom: 1px;
              ${StyledListItemItem} {
                border-bottom-left-radius: var(--disabled-border-radius);
                border-bottom-right-radius: var(--disabled-border-radius);
              }
            }
          `;
        }

        return null;
      }}

      ${() => {
        if ($position === 'middle' || $position === 'last') {
          return css`
            ${StyledListItemItem} {
              border-top-left-radius: var(--disabled-border-radius);
              border-top-right-radius: var(--disabled-border-radius);
            }
          `;
        }

        return null;
      }}

      ${() => {
        if ($position === 'middle' || $position === 'first') {
          return css`
            margin-bottom: 1px;
            ${StyledListItemItem} {
              border-bottom-left-radius: var(--disabled-border-radius);
              border-bottom-right-radius: var(--disabled-border-radius);
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
  level?: keyof typeof levels;
  position?: ListItemPositions;
  disabled?: boolean;
  animation?: ListItemAnimation | false;
  children: React.ReactNode;
} & React.ComponentPropsWithRef<'div'>;

const ListItem = ({
  level = 0,
  position = 'auto',
  disabled,
  animation: animationProp,
  children,
  ...props
}: ListItemProps) => {
  const animation = useListItemAnimation(animationProp);
  return (
    <ListItemProvider level={level} disabled={disabled} animation={animationProp}>
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
  Content: ListItemContent,
  Label: ListItemLabel,
  Group: ListItemGroup,
  Text: ListItemText,
  Trigger: ListItemTrigger,
  Item: ListItemItem,
  Footer: ListItemFooter,
  Icon: ListItemIcon,
  TextField: ListItemTextField,
});

export { ListItemNamespace as ListItem };
