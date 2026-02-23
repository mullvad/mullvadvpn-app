import React from 'react';

import type { AnyLocation } from '../../select-location-types';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';

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
  rootLocation?: 'customList' | 'geographical';
}>;

export function AnyLocationListItemProvider({
  location,
  rootLocation,
  children,
  ...props
}: AnyLocationListItemProviderProps) {
  const [loading, setLoading] = React.useState(false);
  const { searchTerm } = useSelectLocationViewContext();
  const [expanded, setExpanded] = React.useState(location.expanded);

  React.useEffect(() => {
    setExpanded(location.expanded);
  }, [location.expanded, searchTerm]);

  const value = React.useMemo(
    () => ({
      location,
      rootLocation,
      loading,
      setLoading,
      expanded,
      setExpanded,
    }),
    [location, rootLocation, loading, expanded],
  );

  return (
    <AnyLocationListItemContext.Provider value={value} {...props}>
      {children}
    </AnyLocationListItemContext.Provider>
  );
}
