import React from 'react';

type RecentGeographicalLocationContextProps = Omit<
  RecentGeographicalLocationProviderProps,
  'children'
> & {
  loading?: boolean;
  setLoading: React.Dispatch<React.SetStateAction<boolean>>;
};

const RecentGeographicalLocationContext = React.createContext<
  RecentGeographicalLocationContextProps | undefined
>(undefined);

export const useRecentGeographicalLocationContext = (): RecentGeographicalLocationContextProps => {
  const context = React.useContext(RecentGeographicalLocationContext);
  if (!context) {
    throw new Error(
      'useRecentGeographicalLocationContext must be used within a RecentGeographicalLocationProvider',
    );
  }
  return context;
};

type RecentGeographicalLocationProviderProps = React.PropsWithChildren;

export function RecentGeographicalLocationProvider({
  children,
  ...props
}: RecentGeographicalLocationProviderProps) {
  const [loading, setLoading] = React.useState(false);

  const value = React.useMemo(
    () => ({
      loading,
      setLoading,
    }),
    [loading],
  );

  return (
    <RecentGeographicalLocationContext.Provider value={{ ...props, ...value }}>
      {children}
    </RecentGeographicalLocationContext.Provider>
  );
}
