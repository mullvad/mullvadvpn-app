import React from 'react';

type CustomListLocationListItemContextProps = Omit<
  CustomListLocationListItemProviderProps,
  'children'
> & {
  loading?: boolean;
  setLoading: React.Dispatch<React.SetStateAction<boolean>>;
};

const CustomListLocationListItemContext = React.createContext<
  CustomListLocationListItemContextProps | undefined
>(undefined);

export const useCustomListLocationListItemContext = (): CustomListLocationListItemContextProps => {
  const context = React.useContext(CustomListLocationListItemContext);
  if (!context) {
    throw new Error(
      'useCustomListLocationListItemContext must be used within a CustomListLocationListItemProvider',
    );
  }
  return context;
};

type CustomListLocationListItemProviderProps = React.PropsWithChildren;

export function CustomListLocationListItemProvider({
  children,
  ...props
}: CustomListLocationListItemProviderProps) {
  const [loading, setLoading] = React.useState(false);

  const value = React.useMemo(
    () => ({
      loading,
      setLoading,
    }),
    [loading],
  );

  return (
    <CustomListLocationListItemContext.Provider value={{ ...props, ...value }}>
      {children}
    </CustomListLocationListItemContext.Provider>
  );
}
