import React from 'react';

import { ListboxProps } from './Listbox';

type ListboxContext<T> = Pick<ListboxProps<T>, 'value' | 'onValueChange'> & {
  labelId: string;
  optionsRef: React.RefObject<HTMLUListElement | null>;
  focusedIndex?: number;
  setFocusedIndex: React.Dispatch<React.SetStateAction<number | undefined>>;
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
  const [focusedIndex, setFocusedIndex] = React.useState<number | undefined>(undefined);
  const optionsRef = React.useRef<HTMLUListElement>(null);
  return (
    <TypedListboxContext.Provider
      value={{
        value,
        onValueChange,
        labelId,
        focusedIndex,
        setFocusedIndex,
        optionsRef,
      }}>
      {children}
    </TypedListboxContext.Provider>
  );
}
