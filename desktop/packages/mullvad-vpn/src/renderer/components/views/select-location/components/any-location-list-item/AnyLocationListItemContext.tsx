import React from 'react';

import type { AnyLocation } from '../../select-location-types';

type AnyLocationListItemContextProps = Omit<AnyLocationListItemProviderProps, 'children'>;

const AnyLocationListItemContext = React.createContext<AnyLocationListItemContextProps | undefined>(
  undefined,
);

export const useAnyLocationListItemContext = (): AnyLocationListItemContextProps => {
  const context = React.useContext(AnyLocationListItemContext);
  if (!context) {
    throw new Error(
      'useAnyLocationListItemContext must be used within a AnyLocationListItemProvider',
    );
  }
  return context;
};

type AnyLocationListItemProviderProps = React.PropsWithChildren<{
  location: AnyLocation;
}>;

export function AnyLocationListItemProvider({
  location,
  children,
  ...props
}: AnyLocationListItemProviderProps) {
  return (
    <AnyLocationListItemContext.Provider value={{ location, ...props }}>
      {children}
    </AnyLocationListItemContext.Provider>
  );
}
