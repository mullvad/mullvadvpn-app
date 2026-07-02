import React from 'react';

type SelectLocationSelectorContextProps = Omit<SelectLocationSelectorProviderProps, 'children'> & {
  isolatedItem: string | undefined;
  setIsolatedItem: React.Dispatch<React.SetStateAction<string | undefined>>;
};

const SelectLocationSelectorContext = React.createContext<
  SelectLocationSelectorContextProps | undefined
>(undefined);

export const useSelectLocationSelectorContext = (): SelectLocationSelectorContextProps => {
  const context = React.useContext(SelectLocationSelectorContext);
  if (!context) {
    throw new Error(
      'useSelectLocationSelectorContext must be used within a SelectLocationSelectorProvider',
    );
  }
  return context;
};

export type SelectLocationSelectorProviderProps = React.PropsWithChildren;

export function SelectLocationSelectorProvider({ children }: SelectLocationSelectorProviderProps) {
  const [isolatedItem, setIsolatedItem] = React.useState<string | undefined>(undefined);
  const value = React.useMemo(
    () => ({
      isolatedItem,
      setIsolatedItem,
    }),
    [isolatedItem, setIsolatedItem],
  );

  return (
    <SelectLocationSelectorContext.Provider value={value}>
      {children}
    </SelectLocationSelectorContext.Provider>
  );
}
