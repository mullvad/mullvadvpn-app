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
} from './components';
import { useListItemAnimation } from './hooks';
import { levels } from './levels';
import { ListItemProvider } from './ListItemContext';

export type ListItemAnimation = 'flash' | 'dim';

export const StyledListItem = styled.div<{
  $animation?: RuleSet<object>;
}>`
  ${({ $animation }) => {
    return css`
      ${$animation}
    `;
  }}
`;

export type ListItemProps = {
  level?: keyof typeof levels;
  disabled?: boolean;
  animation?: ListItemAnimation | false;
  children: React.ReactNode;
} & React.ComponentPropsWithRef<'div'>;

const ListItem = ({
  level = 0,
  disabled,
  animation: animationProp,
  children,
  ...props
}: ListItemProps) => {
  const animation = useListItemAnimation(animationProp);
  return (
    <ListItemProvider level={level} disabled={disabled} animation={animationProp}>
      <StyledListItem $animation={animationProp == 'dim' ? animation : undefined} {...props}>
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
