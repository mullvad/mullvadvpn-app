import React from 'react';

type LocationListItemContextProps = Omit<LocationListItemProviderProps, 'children'>;

const LocationListItemContext = React.createContext<LocationListItemContextProps | undefined>(
  undefined,
);

export const useLocationListItemContext = (): LocationListItemContextProps => {
  const context = React.useContext(LocationListItemContext);
  if (!context) {
    throw new Error('useLocationListItemContext must be used within a LocationListItemProvider');
  }
  return context;
};

type LocationListItemProviderProps = React.PropsWithChildren<{
  selected?: boolean;
}>;

export function LocationListItemProvider({ children, ...props }: LocationListItemProviderProps) {
  return (
    <LocationListItemContext.Provider value={{ ...props }}>
      {children}
    </LocationListItemContext.Provider>
  );
}
