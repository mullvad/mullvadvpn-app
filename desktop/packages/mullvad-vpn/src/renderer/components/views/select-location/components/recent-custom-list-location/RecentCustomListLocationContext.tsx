import React from 'react';

type RecentCustomListLocationContextProps = Omit<RecentCustomListProviderProps, 'children'> & {
  loading?: boolean;
  setLoading: React.Dispatch<React.SetStateAction<boolean>>;
};

const RecentCustomListContext = React.createContext<
  RecentCustomListLocationContextProps | undefined
>(undefined);

export const useRecentCustomListLocationContext = (): RecentCustomListLocationContextProps => {
  const context = React.useContext(RecentCustomListContext);
  if (!context) {
    throw new Error(
      'useRecentCustomListLocationContext must be used within a RecentCustomListProvider',
    );
  }
  return context;
};

type RecentCustomListProviderProps = React.PropsWithChildren;

export function RecentCustomListProvider({ children, ...props }: RecentCustomListProviderProps) {
  const [loading, setLoading] = React.useState(false);

  const value = React.useMemo(
    () => ({
      loading,
      setLoading,
    }),
    [loading],
  );

  return (
    <RecentCustomListContext.Provider value={{ ...props, ...value }}>
      {children}
    </RecentCustomListContext.Provider>
  );
}
