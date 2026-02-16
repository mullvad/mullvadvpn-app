import React from 'react';

import { useCustomListLocations } from './hooks';
import { type CustomListLocation } from './select-location-types';

type CustomListLocationsContextProps = Omit<CustomListLocationsProviderProps, 'children'> & {
  customListLocations: CustomListLocation[];
};

const CustomListLocationsContext = React.createContext<CustomListLocationsContextProps | undefined>(
  undefined,
);

export const useCustomListLocationsContext = (): CustomListLocationsContextProps => {
  const context = React.useContext(CustomListLocationsContext);
  if (!context) {
    throw new Error(
      'useCustomListLocationsContext must be used within a CustomListLocationsProvider',
    );
  }
  return context;
};

type CustomListLocationsProviderProps = React.PropsWithChildren;

export function CustomListLocationsProvider({ children }: CustomListLocationsProviderProps) {
  const customListLocations = useCustomListLocations();

  const value = React.useMemo(
    () => ({
      customListLocations,
    }),
    [customListLocations],
  );

  return (
    <CustomListLocationsContext.Provider value={value}>
      {children}
    </CustomListLocationsContext.Provider>
  );
}
