import { createContext, useContext } from 'react';

import type { levels } from './levels';
import { ListItemProps } from './ListItem';

type ListItemContextType = {
  level: keyof typeof levels;
  disabled?: boolean;
  animation?: ListItemProps['animation'];
};

const ListItemContext = createContext<ListItemContextType | undefined>(undefined);

type ListItemProviderProps = React.PropsWithChildren<ListItemContextType>;

export const ListItemProvider = ({ children, ...props }: ListItemProviderProps) => {
  return <ListItemContext.Provider value={props}>{children}</ListItemContext.Provider>;
};

export const useListItemContext = (): ListItemContextType => {
  const context = useContext(ListItemContext);
  if (!context) {
    throw new Error('useListItem must be used within a ListItemProvider');
  }
  return context;
};
