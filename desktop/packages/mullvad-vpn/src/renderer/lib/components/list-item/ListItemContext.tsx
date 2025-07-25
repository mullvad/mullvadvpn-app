import { createContext, ReactNode, useContext } from 'react';

import { levels } from './levels';
import { ListItemAnimation } from './ListItem';

interface ListItemContextType {
  level: keyof typeof levels;
  disabled?: boolean;
  animation?: ListItemAnimation;
}

const ListItemContext = createContext<ListItemContextType | undefined>(undefined);

interface ListItemProviderProps extends ListItemContextType {
  animation?: ListItemAnimation;
  children: ReactNode;
}

export const ListItemProvider = ({
  level,
  disabled,
  animation,
  children,
}: ListItemProviderProps) => {
  return (
    <ListItemContext.Provider value={{ level, disabled, animation }}>
      {children}
    </ListItemContext.Provider>
  );
};

export const useListItem = (): ListItemContextType => {
  const context = useContext(ListItemContext);
  if (!context) {
    throw new Error('useListItem must be used within a ListItemProvider');
  }
  return context;
};
