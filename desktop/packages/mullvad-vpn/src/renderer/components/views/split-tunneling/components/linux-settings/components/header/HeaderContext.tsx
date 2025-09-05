import React, { useMemo, useState } from 'react';

type HeaderContextProviderProps = {
  children: React.ReactNode;
};

type HeaderContext = {
  showUnsupportedDialog: boolean;
  setShowUnsupportedDialog: (value: boolean) => void;
};

const HeaderContext = React.createContext<HeaderContext | undefined>(undefined);

export const useHeaderContext = (): HeaderContext => {
  const context = React.useContext(HeaderContext);
  if (!context) {
    throw new Error('useLinuxSettings must be used within a HeaderContext');
  }
  return context;
};

export function HeaderContextProvider({ children }: HeaderContextProviderProps) {
  const [showUnsupportedDialog, setShowUnsupportedDialog] = useState(false);

  const value = useMemo(
    () => ({
      showUnsupportedDialog,
      setShowUnsupportedDialog,
    }),
    [showUnsupportedDialog, setShowUnsupportedDialog],
  );

  return <HeaderContext value={value}>{children}</HeaderContext>;
}
