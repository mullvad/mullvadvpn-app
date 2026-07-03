import React from 'react';

import type { StatusDialogProps } from './StatusDialog';

type StatusDialogContextProps = Omit<StatusDialogProviderProps, 'children'>;

const StatusDialogContext = React.createContext<StatusDialogContextProps | undefined>(undefined);

export const useStatusDialogContext = (): StatusDialogContextProps => {
  const context = React.useContext(StatusDialogContext);
  if (!context) {
    throw new Error('useStatusDialogContext must be used within a StatusDialogProvider');
  }
  return context;
};

type StatusDialogProviderProps = React.PropsWithChildren & Pick<StatusDialogProps, 'variant'>;

export function StatusDialogProvider({ children, variant }: StatusDialogProviderProps) {
  const value = React.useMemo(
    () => ({
      variant,
    }),
    [variant],
  );

  return <StatusDialogContext.Provider value={value}>{children}</StatusDialogContext.Provider>;
}
