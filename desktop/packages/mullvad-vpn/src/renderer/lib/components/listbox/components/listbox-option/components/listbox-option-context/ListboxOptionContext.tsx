import React from 'react';

import { ListboxOptionProps } from '../../ListboxOption';

type ListboxOptionContext<T> = Pick<ListboxOptionProps<T>, 'value'>;

type ListboxOptionProviderProps<T> = ListboxOptionContext<T> & {
  children: React.ReactNode;
};

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

export function ListboxOptionProvider<T>({ value, children }: ListboxOptionProviderProps<T>) {
  const TypedListboxOptionContext = ListboxOptionContext as React.Context<ListboxOptionContext<T>>;
  return (
    <TypedListboxOptionContext.Provider
      value={{
        value,
      }}>
      {children}
    </TypedListboxOptionContext.Provider>
  );
}
