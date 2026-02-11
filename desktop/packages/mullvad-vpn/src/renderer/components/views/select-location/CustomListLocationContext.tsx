import React from 'react';

import { useCustomListsRelayList } from './hooks';
import { useRelayListContext } from './RelayListContext';
import type { CustomListLocation } from './select-location-types';

type CustomListLocationContextProps = Omit<CustomListLocationProviderProps, 'children'> & {
  customListLocations: CustomListLocation[];
};

const CustomListLocationContext = React.createContext<CustomListLocationContextProps | undefined>(
  undefined,
);

export const useCustomListLocationContext = (): CustomListLocationContextProps => {
  const context = React.useContext(CustomListLocationContext);
  if (!context) {
    throw new Error(
      'useCustomListLocationContext must be used within a CustomListLocationProvider',
    );
  }
  return context;
};

type CustomListLocationProviderProps = React.PropsWithChildren;

export function CustomListLocationProvider({ children }: CustomListLocationProviderProps) {
  const { relayList, expandedLocations } = useRelayListContext();
  const customLists = useCustomListsRelayList(relayList, expandedLocations);

  const value = React.useMemo(
    () => ({
      customListLocations: customLists,
    }),
    [customLists],
  );

  return (
    <CustomListLocationContext.Provider value={value}>
      {children}
    </CustomListLocationContext.Provider>
  );
}
