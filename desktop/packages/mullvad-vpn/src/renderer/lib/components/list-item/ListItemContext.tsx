import { createContext, useContext } from 'react';
import React from 'react';

import type { levels } from './levels';
import { type ListItemPositions, ListItemProps } from './ListItem';

type ListItemContextType = {
  level: keyof typeof levels;
  position: ListItemPositions;
  disabled?: boolean;
  animation?: ListItemProps['animation'];
};

const ListItemContext = createContext<ListItemContextType | undefined>(undefined);

type ListItemProviderProps = React.PropsWithChildren<ListItemContextType>;

export const ListItemProvider = ({ children, ...props }: ListItemProviderProps) => {
  const value = React.useMemo(() => props, [props]);
  return <ListItemContext.Provider value={value}>{children}</ListItemContext.Provider>;
};

export const useListItemContext = (): ListItemContextType => {
  const context = useContext(ListItemContext);
  if (!context) {
    throw new Error('useListItemContext must be used within a ListItemProvider');
  }
  return context;
};
