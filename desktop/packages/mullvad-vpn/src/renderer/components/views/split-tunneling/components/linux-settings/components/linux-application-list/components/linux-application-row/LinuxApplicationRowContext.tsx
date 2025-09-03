import React, { useCallback, useMemo, useState } from 'react';

import { type ILinuxSplitTunnelingApplication } from '../../../../../../../../../../shared/application-types';
import { type LinuxApplicationRowProps } from './types';

type LinuxApplicationRowContextProviderProps = LinuxApplicationRowProps & {
  children: React.ReactNode;
};

type LinuxApplicationRowContext = {
  application: ILinuxSplitTunnelingApplication;
  launch: () => void;
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

  const launch = useCallback(() => {
    setShowWarningDialog(false);
    onSelect?.(application);
  }, [onSelect, application]);

  const value = useMemo(
    () => ({
      application,
      launch,
      showWarningDialog,
      setShowWarningDialog,
    }),
    [application, launch, showWarningDialog, setShowWarningDialog],
  );

  return <LinuxApplicationRowContext value={value}>{children}</LinuxApplicationRowContext>;
}
