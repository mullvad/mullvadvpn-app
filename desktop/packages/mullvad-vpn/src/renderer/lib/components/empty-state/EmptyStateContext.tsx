import React from 'react';

import { EmptyStateProps } from './EmptyState';

type EmptyStateContextProps = {
  variant: EmptyStateProps['variant'];
};

const EmptyStateContext = React.createContext<EmptyStateContextProps | undefined>(undefined);

export const useEmptyStateContext = (): EmptyStateContextProps => {
  const context = React.useContext(EmptyStateContext);
  if (!context) {
    throw new Error('useEmptyStateContext must be used within a EmptyStateProvider');
  }
  return context;
};

interface EmptyStateProviderProps {
  variant: EmptyStateProps['variant'];
  children: React.ReactNode;
}

export function EmptyStateProvider({ variant, children }: EmptyStateProviderProps) {
  return <EmptyStateContext.Provider value={{ variant }}>{children}</EmptyStateContext.Provider>;
}
