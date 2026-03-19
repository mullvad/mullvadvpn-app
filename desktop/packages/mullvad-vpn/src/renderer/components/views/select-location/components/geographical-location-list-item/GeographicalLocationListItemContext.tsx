import React from 'react';

type GeographicalLocationListItemContextProps = Omit<
  GeographicalLocationListItemProviderProps,
  'children'
> & {
  loading?: boolean;
  setLoading: React.Dispatch<React.SetStateAction<boolean>>;
};

const GeographicalLocationListItemContext = React.createContext<
  GeographicalLocationListItemContextProps | undefined
>(undefined);

export const useGeographicalLocationListItemContext =
  (): GeographicalLocationListItemContextProps => {
    const context = React.useContext(GeographicalLocationListItemContext);
    if (!context) {
      throw new Error(
        'useGeographicalLocationListItemContext must be used within a GeographicalLocationListItemProvider',
      );
    }
    return context;
  };

type GeographicalLocationListItemProviderProps = React.PropsWithChildren;

export function GeographicalLocationListItemProvider({
  children,
  ...props
}: GeographicalLocationListItemProviderProps) {
  const [loading, setLoading] = React.useState(false);

  const value = React.useMemo(
    () => ({
      loading,
      setLoading,
    }),
    [loading],
  );

  return (
    <GeographicalLocationListItemContext.Provider value={{ ...props, ...value }}>
      {children}
    </GeographicalLocationListItemContext.Provider>
  );
}
