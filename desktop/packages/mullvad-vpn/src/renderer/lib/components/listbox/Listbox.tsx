import React from 'react';

import { ListItem, ListItemProps } from '../list-item';
import { ListboxLabel, ListboxOption, ListboxOptions, ListboxProvider } from './components';

export type ListboxProps<T> = ListItemProps & {
  onValueChange?: (value: T) => Promise<void>;
  value?: T;
};

function Listbox<T>({ value, onValueChange, children, ...props }: ListboxProps<T>) {
  const labelId = React.useId();

  return (
    <ListboxProvider labelId={labelId} value={value} onValueChange={onValueChange}>
      <ListItem {...props}>{children}</ListItem>
    </ListboxProvider>
  );
}

const ListboxNamespace = Object.assign(Listbox, {
  Item: ListItem.Item,
  Content: ListItem.Content,
  Label: ListboxLabel,
  Group: ListItem.Group,
  Text: ListItem.Text,
  Footer: ListItem.Footer,
  Icon: ListItem.Icon,
  Option: ListboxOption,
  Options: ListboxOptions,
});

export { ListboxNamespace as Listbox };
