import React from 'react';
import styled, { css, RuleSet } from 'styled-components';

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
  ListItemTrigger,
  StyledListItemItem,
  StyledListItemTrailingAction,
  StyledListItemTrigger,
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

      display: grid;
      grid-template-columns: 1fr;

      // If it has a trailing action at the end
      &&:has(> ${StyledListItemTrailingAction}, > ${StyledListItemTrigger}:nth-child(2)) {
        grid-template-columns: 1fr auto;
        ${StyledListItemItem} {
          border-top-right-radius: var(--disabled-border-radius);
        }
      }

      ${() => {
        if ($position === 'auto') {
          return css`
            // If directly preceded by another ListItem
            ${StyledListItemRoot} + & {
              ${StyledListItemItem} {
                border-top-left-radius: var(--disabled-border-radius);
                border-top-right-radius: var(--disabled-border-radius);
              }
              ${StyledListItemTrailingAction} {
                border-top-right-radius: var(--disabled-border-radius);
              }
            }

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

      ${() => {
        // Absolutely terrible hack for supporting fractional scaling on Linux under Wayland
        if (window.navigator.platform.includes('Linux')) {
          return css`
            margin-bottom: 1px;

            @media (resolution >= 1.25x) and (resolution < 1.33x) {
              margin-bottom: calc(-1px * 1.25);
              padding-top: 1.75px;
              padding-bottom: calc(0.25px / 1);
            }

            @media (resolution >= 1.33x) and (resolution < 1.5x) {
              margin-bottom: calc(-1px * 1.33);
              padding-top: 1.66px;
              padding-bottom: calc(0.33px / 1);
            }

            @media (resolution >= 1.5x) and (resolution < 1.66x) {
              margin-bottom: calc(-1px * 1.5);
              padding-top: 1.5px;
              padding-bottom: calc(0.5px / 1);
            }

            @media (resolution >= 1.66x) and (resolution < 1.75x) {
              margin-bottom: calc(-1px * 1.66);
              padding-top: 1.66px;
              padding-bottom: calc(1.33px / 1);
            }

            @media (resolution >= 1.75x) and (resolution < 2x) {
              margin-bottom: calc(-1px * 1.75);
              padding-top: 1.75px;
              padding-bottom: calc(1.25px / 1);
            }

            @media (resolution >= 2x) and (resolution < 2.33x) {
              margin-bottom: 1px;
            }

            /** Untested resolution */
            @media (resolution >= 2.33x) and (resolution < 2.5x) {
              margin-bottom: calc(-1px * 2.33);
              padding-top: 2.66px;
              padding-bottom: calc((2.33px - 1px) / 2);
            }

            @media (resolution >= 2.5x) and (resolution < 2.66x) {
              margin-bottom: calc(-1px * 2.5);
              padding-top: 2.5px;
              padding-bottom: calc(1.5px / 2);
            }

            @media (resolution >= 2.66x) and (resolution < 2.75x) {
              margin-bottom: calc(-1px * 2.66);
              padding-top: 2.66px;
              padding-bottom: calc(2.33px / 2);
            }

            @media (resolution >= 2.75x) and (resolution < 3x) {
              margin-bottom: calc(-1px * 2.75);
              padding-top: 2.75px;
              padding-bottom: calc(2.25px / 2);
            }
          `;
        }

        return css`
          margin-bottom: 1px;
        `;
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
  TrailingAction: ListItemTrailingAction,
});

export { ListItemNamespace as ListItem };
