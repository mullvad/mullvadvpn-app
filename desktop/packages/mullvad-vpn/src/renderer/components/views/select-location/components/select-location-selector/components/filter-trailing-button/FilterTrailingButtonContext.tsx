import React from 'react';

import { type FilterTrailingButtonProps } from './FilterTrailingButton';

type FilterTrailingButtonContextProps = FilterTrailingButtonProps;

const FilterTrailingButtonContext = React.createContext<
  FilterTrailingButtonContextProps | undefined
>(undefined);

export const useFilterTrailingButtonContext = (): FilterTrailingButtonContextProps => {
  const context = React.useContext(FilterTrailingButtonContext);
  if (!context) {
    throw new Error(
      'useFilterTrailingButtonContext must be used within a FilterTrailingButtonProvider',
    );
  }
  return context;
};

export type FilterTrailingButtonProviderProps = React.PropsWithChildren<FilterTrailingButtonProps>;

export function FilterTrailingButtonProvider({
  type,
  children,
}: FilterTrailingButtonProviderProps) {
  const value = React.useMemo(
    () => ({
      type,
    }),
    [type],
  );

  return (
    <FilterTrailingButtonContext.Provider value={value}>
      {children}
    </FilterTrailingButtonContext.Provider>
  );
}
