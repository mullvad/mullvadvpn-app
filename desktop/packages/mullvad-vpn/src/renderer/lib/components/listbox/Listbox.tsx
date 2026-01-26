import React from 'react';

import { ListItem } from '../list-item';
import {
  ListboxFooter,
  ListboxFooterText,
  ListboxHeader,
  ListboxLabel,
  ListboxOption,
  ListboxOptions,
} from './components';
import { ListboxProvider } from './ListboxContext';

export type ListboxProps<T> = React.PropsWithChildren<{
  onValueChange?: ((value: T) => Promise<void>) | ((value: T) => void);
  value?: T;
  labelId?: string;
}>;

function Listbox<T>({ value, onValueChange, labelId: labelIdProp, children }: ListboxProps<T>) {
  const labelId = React.useId();

  return (
    <ListboxProvider labelId={labelIdProp ?? labelId} value={value} onValueChange={onValueChange}>
      <div tabIndex={-1} role="region" aria-labelledby={labelIdProp ?? labelId}>
        {children}
      </div>
    </ListboxProvider>
  );
}

const ListboxNamespace = Object.assign(Listbox, {
  Header: ListboxHeader,
  HeaderItem: ListItem.Item,
  Label: ListboxLabel,
  Group: ListItem.Group,
  ActionGroup: ListItem.ActionGroup,
  Text: ListItem.Text,
  Footer: ListboxFooter,
  FooterText: ListboxFooterText,
  Icon: ListItem.Icon,
  Option: ListboxOption,
  Options: ListboxOptions,
});

export { ListboxNamespace as Listbox };
