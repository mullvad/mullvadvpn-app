import React from 'react';

type CustomListLocationListContextProps = Omit<CustomListLocationListProviderProps, 'children'> & {
  loading?: boolean;
  setLoading: React.Dispatch<React.SetStateAction<boolean>>;
};

const CustomListLocationListContext = React.createContext<
  CustomListLocationListContextProps | undefined
>(undefined);

export const useCustomListLocationListContext = (): CustomListLocationListContextProps => {
  const context = React.useContext(CustomListLocationListContext);
  if (!context) {
    throw new Error(
      'useCustomListLocationListContext must be used within a CustomListLocationListProvider',
    );
  }
  return context;
};

type CustomListLocationListProviderProps = React.PropsWithChildren;

export function CustomListLocationListProvider({
  children,
  ...props
}: CustomListLocationListProviderProps) {
  const [loading, setLoading] = React.useState(false);

  const value = React.useMemo(
    () => ({
      loading,
      setLoading,
    }),
    [loading],
  );

  return (
    <CustomListLocationListContext.Provider value={{ ...props, ...value }}>
      {children}
    </CustomListLocationListContext.Provider>
  );
}
