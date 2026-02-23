import React from 'react';

import { useListItemContext } from '../../ListItemContext';

type ListItemTriggerContext = Omit<ListItemTriggerProviderProps, 'children'>;

const ListItemTriggerContext = React.createContext<ListItemTriggerContext | undefined>(undefined);

export const useListItemTriggerContext = (): ListItemTriggerContext => {
  const context = React.useContext(ListItemTriggerContext);
  if (!context) {
    throw new Error('useListItemTriggerContext must be used within a ListItemTriggerProvider');
  }
  return context;
};

type ListItemTriggerProviderProps = React.PropsWithChildren<{
  disabled?: boolean;
}>;

export function ListItemTriggerProvider({
  disabled: disabledProp,
  children,
  ...props
}: ListItemTriggerProviderProps) {
  const { disabled: disabledContext } = useListItemContext();
  const disabled = disabledProp ?? disabledContext;
  return (
    <ListItemTriggerContext.Provider value={{ disabled, ...props }}>
      {children}
    </ListItemTriggerContext.Provider>
  );
}
