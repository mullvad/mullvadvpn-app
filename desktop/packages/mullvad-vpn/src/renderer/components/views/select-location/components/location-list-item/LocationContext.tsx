import React from 'react';

type LocationContextProps = Omit<LocationProviderProps, 'children'>;

const LocationContext = React.createContext<LocationContextProps | undefined>(undefined);

export const useLocationContext = (): LocationContextProps => {
  const context = React.useContext(LocationContext);
  if (!context) {
    throw new Error('useLocationContext must be used within a LocationProvider');
  }
  return context;
};

type LocationProviderProps = React.PropsWithChildren<{
  root?: boolean;
  selected?: boolean;
}>;

export function LocationProvider({ children, ...props }: LocationProviderProps) {
  return <LocationContext.Provider value={{ ...props }}>{children}</LocationContext.Provider>;
}
