import React from 'react';

import {
  ListItemContent,
  ListItemFooter,
  ListItemGroup,
  ListItemIcon,
  ListItemItem,
  ListItemLabel,
  ListItemText,
  ListItemTrigger,
} from './components';
import { levels } from './levels';
import { ListItemProvider } from './ListItemContext';

export type ListItemAnimation = 'flash' | 'dim';

export type ListItemProps = {
  level?: keyof typeof levels;
  disabled?: boolean;
  animation?: ListItemAnimation;
  children: React.ReactNode;
};

const ListItem = ({ level = 0, disabled, animation, children }: ListItemProps) => {
  return (
    <ListItemProvider level={level} disabled={disabled} animation={animation}>
      {children}
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
});

export { ListItemNamespace as ListItem };
