import React from 'react';

import { useBoolean } from '../../../../../lib/utility-hooks';
import type { LocationSelection } from './CustomLists';

type CustomListsContextProps = Omit<CustomListsProviderProps, 'children'> & {
  addFormVisible: boolean;
  showAddForm: () => void;
  hideAddForm: () => void;
};

const CustomListsContext = React.createContext<CustomListsContextProps | undefined>(undefined);

export const useCustomListsContext = (): CustomListsContextProps => {
  const context = React.useContext(CustomListsContext);
  if (!context) {
    throw new Error('useCustomListsContext must be used within a CustomListsProvider');
  }
  return context;
};

type CustomListsProviderProps = React.PropsWithChildren & {
  locationSelection: LocationSelection;
};

export function CustomListsProvider({ children, ...props }: CustomListsProviderProps) {
  const [addFormVisible, showAddForm, hideAddForm] = useBoolean(false);

  const value = React.useMemo(
    () => ({
      addFormVisible,
      showAddForm,
      hideAddForm,
      ...props,
    }),
    [addFormVisible, showAddForm, hideAddForm, props],
  );

  return <CustomListsContext.Provider value={value}>{children}</CustomListsContext.Provider>;
}
