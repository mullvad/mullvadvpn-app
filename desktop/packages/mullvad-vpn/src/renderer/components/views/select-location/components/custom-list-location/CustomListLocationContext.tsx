import React from 'react';

type CustomListLocationContextProps = Omit<CustomListLocationProviderProps, 'children'> & {
  loading?: boolean;
  setLoading: React.Dispatch<React.SetStateAction<boolean>>;
};

const CustomListLocationContext = React.createContext<CustomListLocationContextProps | undefined>(
  undefined,
);

export const useCustomListLocationContext = (): CustomListLocationContextProps => {
  const context = React.useContext(CustomListLocationContext);
  if (!context) {
    throw new Error(
      'useCustomListLocationContext must be used within a CustomListLocationProvider',
    );
  }
  return context;
};

type CustomListLocationProviderProps = React.PropsWithChildren;

export function CustomListLocationProvider({
  children,
  ...props
}: CustomListLocationProviderProps) {
  const [loading, setLoading] = React.useState(false);

  const value = React.useMemo(
    () => ({
      loading,
      setLoading,
    }),
    [loading],
  );

  return (
    <CustomListLocationContext.Provider value={{ ...props, ...value }}>
      {children}
    </CustomListLocationContext.Provider>
  );
}
