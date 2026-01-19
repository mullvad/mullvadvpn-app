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

  const handleOnValueChange = React.useCallback(
    async (newValue: T) => {
      if (!onValueChange) {
        return;
      }

      const canSelectMultiple = Array.isArray(value);
      const newValueIsArray = Array.isArray(newValue);
      if (!canSelectMultiple) {
        await onValueChange(newValue);
        return;
      } else if (canSelectMultiple && !newValueIsArray) {
        if (value.includes(newValue)) {
          const nextValue: T[] = value.filter((v) => v !== newValue);
          await onValueChange(nextValue as T);
          return;
        } else {
          const nextValue: T[] = [...value, newValue];
          await onValueChange(nextValue as T);
          return;
        }
      } else if (canSelectMultiple && newValueIsArray) {
        await onValueChange(newValue);
      }
    },
    [onValueChange, value],
  );

  return (
    <TypedListboxContext.Provider
      value={{
        value,
        onValueChange: handleOnValueChange,
        labelId,
        focusedIndex,
        setFocusedIndex,
        optionsRef,
      }}>
      {children}
    </TypedListboxContext.Provider>
  );
}
