import React from 'react';

import { ListItem, ListItemProps } from '../list-item';
import { ListboxLabel, ListboxOption, ListboxOptions } from './components';
import { ListboxProvider } from './ListboxContext';

export type ListboxProps<T> = ListItemProps & {
  onValueChange?: (value: T) => Promise<void>;
  value?: T;
  labelId?: string;
};

function Listbox<T>({
  value,
  onValueChange,
  labelId: labelIdProp,
  children,
  ...props
}: ListboxProps<T>) {
  const labelId = React.useId();

  return (
    <ListboxProvider labelId={labelIdProp ?? labelId} value={value} onValueChange={onValueChange}>
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
