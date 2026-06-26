import React from 'react';

import { type LocationSelectorSelectedItem } from '../../../../../lib/components/location-selector';

type SelectLocationSelectorContextProps = Omit<SelectLocationSelectorProviderProps, 'children'> & {
  isolatedItem: LocationSelectorSelectedItem | undefined;
  setIsolatedItem: React.Dispatch<React.SetStateAction<LocationSelectorSelectedItem | undefined>>;
};

const SelectLocationSelectorContext = React.createContext<
  SelectLocationSelectorContextProps | undefined
>(undefined);

export const useSelectLocationSelectorContext = (): SelectLocationSelectorContextProps => {
  const context = React.useContext(SelectLocationSelectorContext);
  if (!context) {
    throw new Error(
      'useSelectLocationSelectorContext must be used within a SelectLocationSelectorProvider',
    );
  }
  return context;
};

export type SelectLocationSelectorProviderProps = React.PropsWithChildren;

export function SelectLocationSelectorProvider({ children }: SelectLocationSelectorProviderProps) {
  const [isolatedItem, setIsolatedItem] = React.useState<LocationSelectorSelectedItem | undefined>(
    undefined,
  );
  const value = React.useMemo(
    () => ({
      isolatedItem,
      setIsolatedItem,
    }),
    [isolatedItem, setIsolatedItem],
  );

  return (
    <SelectLocationSelectorContext.Provider value={value}>
      {children}
    </SelectLocationSelectorContext.Provider>
  );
}
