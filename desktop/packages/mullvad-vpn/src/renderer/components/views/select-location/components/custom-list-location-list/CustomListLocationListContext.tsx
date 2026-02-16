import React from 'react';

import { useBoolean } from '../../../../../lib/utility-hooks';
import type { LocationSelection } from './CustomListLocationList';

type CustomListLocationListContextProps = Omit<CustomListLocationListProviderProps, 'children'> & {
  addFormVisible: boolean;
  addingForm: boolean;
  setAddingForm: React.Dispatch<React.SetStateAction<boolean>>;
  showAddForm: () => void;
  hideAddForm: () => void;
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

type CustomListLocationListProviderProps = React.PropsWithChildren & {
  locationSelection: LocationSelection;
};

export function CustomListLocationListProvider({
  children,
  ...props
}: CustomListLocationListProviderProps) {
  const [addFormVisible, showAddForm, hideAddForm] = useBoolean(false);
  const [addingForm, setAddingForm] = React.useState(false);

  const value = React.useMemo(
    () => ({
      addFormVisible,
      showAddForm,
      hideAddForm,
      addingForm,
      setAddingForm,
      ...props,
    }),
    [addFormVisible, showAddForm, hideAddForm, addingForm, setAddingForm, props],
  );

  return (
    <CustomListLocationListContext.Provider value={value}>
      {children}
    </CustomListLocationListContext.Provider>
  );
}
