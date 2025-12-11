import React from 'react';

import { useListboxContext } from '../../ListboxContext';
import { ListboxOptionProps } from './ListboxOption';

type ListboxOptionContext<T> = ListboxOptionProviderProps<T> & {
  selected: boolean;
};

type ListboxOptionProviderProps<T> = React.PropsWithChildren<Pick<ListboxOptionProps<T>, 'value'>>;

const ListboxOptionContext = React.createContext<ListboxOptionContext<unknown> | undefined>(
  undefined,
);

export function useListboxOptionContext<T>(): ListboxOptionContext<T> {
  const context = React.useContext(ListboxOptionContext) as ListboxOptionContext<T> | undefined;
  if (!context) {
    throw new Error('useListboxOptionContext must be used within a ListboxOptionProvider');
  }
  return context;
}

export function ListboxOptionProvider<T>({ children, value }: ListboxOptionProviderProps<T>) {
  const TypedListboxOptionContext = ListboxOptionContext as React.Context<ListboxOptionContext<T>>;
  const { value: selectedValue } = useListboxContext();
  const selected = value === selectedValue;

  return (
    <TypedListboxOptionContext.Provider value={{ value, selected }}>
      {children}
    </TypedListboxOptionContext.Provider>
  );
}
