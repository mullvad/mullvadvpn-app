import React from 'react';

import type { AnyLocation } from '../../select-location-types';

type AnyLocationListItemContextProps = Omit<AnyLocationListItemProviderProps, 'children'> & {
  loading: boolean;
  setLoading: React.Dispatch<React.SetStateAction<boolean>>;
  expanded: boolean;
  setExpanded: React.Dispatch<React.SetStateAction<boolean>>;
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
}>;

export function AnyLocationListItemProvider({
  location,
  children,
  ...props
}: AnyLocationListItemProviderProps) {
  const [loading, setLoading] = React.useState(false);
  const [expanded, setExpanded] = React.useState(location.expanded ?? false);

  const value = React.useMemo(
    () => ({
      location,
      loading,
      setLoading,
      expanded,
      setExpanded,
    }),
    [location, loading, expanded],
  );

  return (
    <AnyLocationListItemContext.Provider value={value} {...props}>
      {children}
    </AnyLocationListItemContext.Provider>
  );
}
