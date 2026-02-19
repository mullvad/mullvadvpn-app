import React from 'react';

type CustomListLocationListContextProps = Omit<CustomListLocationListProviderProps, 'children'> & {
  addingCustomList: boolean;
  setAddingCustomList: React.Dispatch<React.SetStateAction<boolean>>;
  addCustomListDialogOpen: boolean;
  setAddCustomListDialogOpen: React.Dispatch<React.SetStateAction<boolean>>;
};

const CustomListLocationListContext = React.createContext<
  CustomListLocationListContextProps | undefined
>(undefined);

export const useCustomListListContext = (): CustomListLocationListContextProps => {
  const context = React.useContext(CustomListLocationListContext);
  if (!context) {
    throw new Error(
      'useCustomListListContext must be used within a CustomListLocationListProvider',
    );
  }
  return context;
};

type CustomListLocationListProviderProps = React.PropsWithChildren;

export function CustomListLocationListProvider({ children }: CustomListLocationListProviderProps) {
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
    <CustomListLocationListContext.Provider value={value}>
      {children}
    </CustomListLocationListContext.Provider>
  );
}
