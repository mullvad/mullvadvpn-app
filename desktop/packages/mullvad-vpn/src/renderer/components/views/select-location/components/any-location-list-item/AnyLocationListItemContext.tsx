import React from 'react';

import type { AnyLocation } from '../../../../../features/locations/types';

type AnyLocationListItemContextProps = Omit<AnyLocationListItemProviderProps, 'children'> & {
  loading: boolean;
  setLoading: React.Dispatch<React.SetStateAction<boolean>>;
};

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
  rootLocation?: 'customList' | 'geographical';
}>;

export function AnyLocationListItemProvider({
  location,
  rootLocation,
  children,
  ...props
}: AnyLocationListItemProviderProps) {
  const [loading, setLoading] = React.useState(false);

  const value = React.useMemo(
    () => ({
      location,
      rootLocation,
      loading,
      setLoading,
    }),
    [location, rootLocation, loading],
  );

  return (
    <AnyLocationListItemContext.Provider value={value} {...props}>
      {children}
    </AnyLocationListItemContext.Provider>
  );
}
