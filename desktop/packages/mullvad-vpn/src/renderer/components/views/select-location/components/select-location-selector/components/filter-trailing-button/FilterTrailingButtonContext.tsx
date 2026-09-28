import React from 'react';

import { type LocationSelectorSelectedItem } from '../../../../../../../lib/components/location-selector';

type FilterTrailingButtonContextProps = Omit<FilterTrailingButtonProviderProps, 'children'> & {
  id: LocationSelectorSelectedItem;
};

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

export type FilterTrailingButtonProviderProps = React.PropsWithChildren<{
  id: LocationSelectorSelectedItem;
}>;

export function FilterTrailingButtonProvider({ id, children }: FilterTrailingButtonProviderProps) {
  const value = React.useMemo(
    () => ({
      id,
    }),
    [id],
  );

  return (
    <FilterTrailingButtonContext.Provider value={value}>
      {children}
    </FilterTrailingButtonContext.Provider>
  );
}
