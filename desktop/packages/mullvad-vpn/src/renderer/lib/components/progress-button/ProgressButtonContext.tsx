import React from 'react';

import type { ProgressButtonStatus } from './ProgressButton';

type ProgressButtonContextProps = Omit<ProgressButtonProviderProps, 'children'>;

const ProgressButtonContext = React.createContext<ProgressButtonContextProps | undefined>(
  undefined,
);

export const useProgressButtonContext = (): ProgressButtonContextProps => {
  const context = React.useContext(ProgressButtonContext);
  if (!context) {
    throw new Error('useProgressButtonContext must be used within a ProgressButtonProvider');
  }
  return context;
};

type ProgressButtonProviderProps = React.PropsWithChildren<{
  status: ProgressButtonStatus;
}>;

export function ProgressButtonProvider({ status, children }: ProgressButtonProviderProps) {
  return (
    <ProgressButtonContext.Provider
      value={{
        status,
      }}>
      {children}
    </ProgressButtonContext.Provider>
  );
}
