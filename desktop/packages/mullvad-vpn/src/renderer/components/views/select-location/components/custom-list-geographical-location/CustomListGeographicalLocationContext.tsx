import React from 'react';

import type { CustomListGeographicalLocationProps } from './CustomListGeographicalLocation';

type CustomListGeographicalLocationContextProps = Omit<
  CustomListGeographicalLocationProviderProps,
  'children'
> & {
  loading?: boolean;
  setLoading: React.Dispatch<React.SetStateAction<boolean>>;
};

const CustomListGeographicalLocationContext = React.createContext<
  CustomListGeographicalLocationContextProps | undefined
>(undefined);

export const useCustomListGeographicalLocationContext =
  (): CustomListGeographicalLocationContextProps => {
    const context = React.useContext(CustomListGeographicalLocationContext);
    if (!context) {
      throw new Error(
        'useCustomListGeographicalLocationContext must be used within a CustomListGeographicalLocationProvider',
      );
    }
    return context;
  };

type CustomListGeographicalLocationProviderProps = React.PropsWithChildren &
  Pick<CustomListGeographicalLocationProps, 'location' | 'level'>;

export function CustomListGeographicalLocationProvider({
  children,
  ...props
}: CustomListGeographicalLocationProviderProps) {
  const [loading, setLoading] = React.useState(false);

  const value = React.useMemo(
    () => ({
      loading,
      setLoading,
    }),
    [loading],
  );

  return (
    <CustomListGeographicalLocationContext.Provider value={{ ...props, ...value }}>
      {children}
    </CustomListGeographicalLocationContext.Provider>
  );
}
