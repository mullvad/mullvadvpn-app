import React from 'react';

import type { LocationSelectorRowPropsPositions } from './LocationSelectorRow';

type LocationSelectorRowContextProps = Omit<LocationSelectorRowProviderProps, 'children'> & {};

const LocationSelectorRowContext = React.createContext<LocationSelectorRowContextProps | undefined>(
  undefined,
);

export const useLocationSelectorRowContext = (): LocationSelectorRowContextProps => {
  const context = React.useContext(LocationSelectorRowContext);
  if (!context) {
    throw new Error(
      'useLocationSelectorRowContext must be used within a LocationSelectorRowProvider',
    );
  }
  return context;
};

type LocationSelectorRowProviderProps = React.PropsWithChildren<{
  position: LocationSelectorRowPropsPositions;
}>;

export function LocationSelectorRowProvider({
  position,
  children,
  ...props
}: LocationSelectorRowProviderProps) {
  const value = React.useMemo(() => ({ position, ...props }), [position, props]);
  return (
    <LocationSelectorRowContext.Provider value={value}>
      {children}
    </LocationSelectorRowContext.Provider>
  );
}
