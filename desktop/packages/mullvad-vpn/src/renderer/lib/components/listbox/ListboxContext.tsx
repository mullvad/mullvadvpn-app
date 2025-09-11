import React from 'react';

import { ListboxProps } from './Listbox';

type ListboxContext<T> = Pick<ListboxProps<T>, 'value' | 'onValueChange'> & {
  labelId: string;
  focusedValue?: T;
  setFocusedValue: React.Dispatch<React.SetStateAction<T | undefined>>;
};

type ListboxProviderProps<T> = Pick<ListboxContext<T>, 'value' | 'onValueChange' | 'labelId'> & {
  children: React.ReactNode;
};

const ListboxContext = React.createContext<ListboxContext<unknown> | undefined>(undefined);

export function useListboxContext<T>(): ListboxContext<T> {
  const context = React.useContext(ListboxContext) as ListboxContext<T> | undefined;
  if (!context) {
    throw new Error('useListboxContext must be used within a ListboxProvider');
  }
  return context;
}

export function ListboxProvider<T>({
  value,
  onValueChange,
  labelId,
  children,
}: ListboxProviderProps<T>) {
  const TypedListboxContext = ListboxContext as React.Context<ListboxContext<T>>;
  const [focusedValue, setFocusedValue] = React.useState<T | undefined>(undefined);
  return (
    <TypedListboxContext.Provider
      value={{
        value,
        onValueChange,
        labelId,
        focusedValue,
        setFocusedValue,
      }}>
      {children}
    </TypedListboxContext.Provider>
  );
}
