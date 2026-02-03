import React from 'react';

import type { LocationSpecification } from '../../select-location-types';

type LocationRowContextProps = Omit<LocationRowProviderProps, 'children'>;

const LocationRowContext = React.createContext<LocationRowContextProps | undefined>(undefined);

export const useLocationRowContext = (): LocationRowContextProps => {
  const context = React.useContext(LocationRowContext);
  if (!context) {
    throw new Error('useLocationRowContext must be used within a LocationRowProvider');
  }
  return context;
};

type LocationRowProviderProps = React.PropsWithChildren<{
  source: LocationSpecification;
}>;

export function LocationRowProvider({ source, children, ...props }: LocationRowProviderProps) {
  return (
    <LocationRowContext.Provider value={{ source, ...props }}>
      {children}
    </LocationRowContext.Provider>
  );
}
