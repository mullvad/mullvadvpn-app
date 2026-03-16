import React from 'react';

import {
  ListboxFooter,
  ListboxFooterText,
  ListboxHeader,
  ListboxOption,
  ListboxOptions,
} from './components';
import { ListboxProvider } from './ListboxContext';

export type ListboxProps<T> = React.PropsWithChildren<{
  value?: T;
  onValueChange?: (value: T) => Promise<void> | void;
  labelId?: string;
}>;

function Listbox<T>({ value, onValueChange, labelId: labelIdProp, children }: ListboxProps<T>) {
  const labelId = React.useId();

  return (
    <ListboxProvider value={value} onValueChange={onValueChange} labelId={labelIdProp ?? labelId}>
      <div tabIndex={-1} role="region" aria-labelledby={labelIdProp ?? labelId}>
        {children}
      </div>
    </ListboxProvider>
  );
}

const ListboxNamespace = Object.assign(Listbox, {
  Header: ListboxHeader,
  Footer: ListboxFooter,
  FooterText: ListboxFooterText,
  Option: ListboxOption,
  Options: ListboxOptions,
});

export { ListboxNamespace as Listbox };
