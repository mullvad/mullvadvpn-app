import React from 'react';

type CustomListLocationListsContextProps = Omit<
  CustomListLocationListsProviderProps,
  'children'
> & {
  addingCustomList: boolean;
  setAddingCustomList: React.Dispatch<React.SetStateAction<boolean>>;
  addCustomListDialogOpen: boolean;
  setAddCustomListDialogOpen: React.Dispatch<React.SetStateAction<boolean>>;
};

const CustomListLocationListsContext = React.createContext<
  CustomListLocationListsContextProps | undefined
>(undefined);

export const useCustomListLocationListsContext = (): CustomListLocationListsContextProps => {
  const context = React.useContext(CustomListLocationListsContext);
  if (!context) {
    throw new Error(
      'useCustomListLocationListsContext must be used within a CustomListLocationListsProvider',
    );
  }
  return context;
};

type CustomListLocationListsProviderProps = React.PropsWithChildren;

export function CustomListLocationListsProvider({
  children,
}: CustomListLocationListsProviderProps) {
  const [addingCustomList, setAddingCustomList] = React.useState(false);
  const [addCustomListDialogOpen, setAddCustomListDialogOpen] = React.useState(false);

  const value = React.useMemo(
    () => ({
      addingCustomList,
      setAddingCustomList,
      addCustomListDialogOpen,
      setAddCustomListDialogOpen,
    }),
    [addCustomListDialogOpen, addingCustomList],
  );

  return (
    <CustomListLocationListsContext.Provider value={value}>
      {children}
    </CustomListLocationListsContext.Provider>
  );
}
