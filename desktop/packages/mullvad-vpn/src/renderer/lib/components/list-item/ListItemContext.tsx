import { createContext, ReactNode, useContext } from 'react';

import { levels } from './levels';

interface ListItemContextType {
  level: keyof typeof levels;
  disabled?: boolean;
}

const ListItemContext = createContext<ListItemContextType | undefined>(undefined);

interface ListItemProviderProps extends ListItemContextType {
  children: ReactNode;
}

export const ListItemProvider = ({ level, disabled, children }: ListItemProviderProps) => {
  return (
    <ListItemContext.Provider value={{ level, disabled }}>{children}</ListItemContext.Provider>
  );
};

export const useListItem = (): ListItemContextType => {
  const context = useContext(ListItemContext);
  if (!context) {
    throw new Error('useListItem must be used within a ListItemProvider');
  }
  return context;
};
