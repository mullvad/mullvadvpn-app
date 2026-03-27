import React from 'react';

type CustomListLocationsContextProps = Omit<CustomListLocationsProviderProps, 'children'> & {
  addingCustomList: boolean;
  setAddingCustomList: React.Dispatch<React.SetStateAction<boolean>>;
  addCustomListDialogOpen: boolean;
  setAddCustomListDialogOpen: React.Dispatch<React.SetStateAction<boolean>>;
};

const CustomListLocationsContext = React.createContext<CustomListLocationsContextProps | undefined>(
  undefined,
);

export const useCustomListLocationsContext = (): CustomListLocationsContextProps => {
  const context = React.useContext(CustomListLocationsContext);
  if (!context) {
    throw new Error(
      'useCustomListLocationsContext must be used within a CustomListLocationsProvider',
    );
  }
  return context;
};

type CustomListLocationsProviderProps = React.PropsWithChildren;

export function CustomListLocationsProvider({ children }: CustomListLocationsProviderProps) {
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
    <CustomListLocationsContext.Provider value={value}>
      {children}
    </CustomListLocationsContext.Provider>
  );
}
