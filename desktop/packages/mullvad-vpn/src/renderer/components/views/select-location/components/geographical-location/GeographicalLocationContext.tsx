import React from 'react';

type GeographicalLocationContextProps = Omit<GeographicalLocationProviderProps, 'children'> & {
  loading?: boolean;
  setLoading: React.Dispatch<React.SetStateAction<boolean>>;
};

const GeographicalLocationContext = React.createContext<
  GeographicalLocationContextProps | undefined
>(undefined);

export const useGeographicalLocationContext = (): GeographicalLocationContextProps => {
  const context = React.useContext(GeographicalLocationContext);
  if (!context) {
    throw new Error(
      'useGeographicalLocationContext must be used within a GeographicalLocationProvider',
    );
  }
  return context;
};

type GeographicalLocationProviderProps = React.PropsWithChildren;

export function GeographicalLocationProvider({
  children,
  ...props
}: GeographicalLocationProviderProps) {
  const [loading, setLoading] = React.useState(false);

  const value = React.useMemo(
    () => ({
      loading,
      setLoading,
    }),
    [loading],
  );

  return (
    <GeographicalLocationContext.Provider value={{ ...props, ...value }}>
      {children}
    </GeographicalLocationContext.Provider>
  );
}
