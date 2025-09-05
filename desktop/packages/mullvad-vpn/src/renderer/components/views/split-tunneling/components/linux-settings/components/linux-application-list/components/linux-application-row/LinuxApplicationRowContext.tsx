import React, { useMemo, useState } from 'react';

import { type LinuxApplicationRowProps } from './LinuxApplicationRow';

type LinuxApplicationRowContextProviderProps = LinuxApplicationRowProps & {
  children: React.ReactNode;
};

type LinuxApplicationRowContext = LinuxApplicationRowProps & {
  showWarningDialog: boolean;
  setShowWarningDialog: (value: boolean) => void;
};

const LinuxApplicationRowContext = React.createContext<LinuxApplicationRowContext | undefined>(
  undefined,
);

export const useLinuxApplicationRowContext = (): LinuxApplicationRowContext => {
  const context = React.useContext(LinuxApplicationRowContext);
  if (!context) {
    throw new Error('useLinuxApplicationRow must be used within a LinuxApplicationRowProvider');
  }
  return context;
};

export function LinuxApplicationRowContextProvider({
  application,
  children,
  onSelect,
}: LinuxApplicationRowContextProviderProps) {
  const [showWarningDialog, setShowWarningDialog] = useState(false);

  const value = useMemo(
    () => ({
      application,
      showWarningDialog,
      setShowWarningDialog,
      onSelect,
    }),
    [application, onSelect, showWarningDialog, setShowWarningDialog],
  );

  return <LinuxApplicationRowContext value={value}>{children}</LinuxApplicationRowContext>;
}
