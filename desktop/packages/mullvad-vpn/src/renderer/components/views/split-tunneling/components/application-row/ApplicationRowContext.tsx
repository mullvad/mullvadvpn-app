import React, { useMemo } from 'react';

import { type ApplicationRowProps } from './ApplicationRow';

type ApplicationRowContextProviderProps = ApplicationRowProps & {
  children: React.ReactNode;
};

type ApplicationRowContext = ApplicationRowProps;

const ApplicationRowContext = React.createContext<ApplicationRowContext | undefined>(undefined);

export const useApplicationRowContext = (): ApplicationRowContext => {
  const context = React.useContext(ApplicationRowContext);
  if (!context) {
    throw new Error('useApplicationRow must be used within a ApplicationRowContext');
  }
  return context;
};

export function ApplicationRowContextProvider({
  application,
  children,
  onAdd,
  onDelete,
  onRemove,
}: ApplicationRowContextProviderProps) {
  const value = useMemo(
    () => ({
      application,
      onAdd,
      onDelete,
      onRemove,
    }),
    [application, onAdd, onDelete, onRemove],
  );

  return <ApplicationRowContext value={value}>{children}</ApplicationRowContext>;
}
