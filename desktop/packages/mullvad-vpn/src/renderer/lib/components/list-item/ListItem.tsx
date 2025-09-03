import React from 'react';

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
import { levels } from './levels';
import { ListItemProvider } from './ListItemContext';

export type ListItemAnimation = 'flash' | 'dim';

export type ListItemProps = {
  level?: keyof typeof levels;
  disabled?: boolean;
  animation?: ListItemAnimation;
  children: React.ReactNode;
} & React.ComponentPropsWithRef<'div'>;

const ListItem = ({ level = 0, disabled, animation, children, ...props }: ListItemProps) => {
  return (
    <ListItemProvider level={level} disabled={disabled} animation={animation}>
      <div {...props}>{children}</div>
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
