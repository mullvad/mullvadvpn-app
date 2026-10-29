import React from 'react';

import { type HeaderMenuProps } from './HeaderMenu';

type HeaderMenuContextProps = Omit<HeaderMenuProviderProps, 'children'> & {
  disableRecentsDialogOpen: boolean;
  setDisableRecentsDialogOpen: React.Dispatch<React.SetStateAction<boolean>>;
};

const HeaderMenuContext = React.createContext<HeaderMenuContextProps | undefined>(undefined);

export const useHeaderMenuContext = (): HeaderMenuContextProps => {
  const context = React.useContext(HeaderMenuContext);
  if (!context) {
    throw new Error('useHeaderMenuContext must be used within a HeaderMenuProvider');
  }
  return context;
};

type HeaderMenuProviderProps = React.PropsWithChildren<HeaderMenuProps>;

export function HeaderMenuProvider({
  children,
  triggerRef,
  onOpenChange,
  open,
}: HeaderMenuProviderProps) {
  const [disableRecentsDialogOpen, setDisableRecentsDialogOpen] = React.useState(false);

  const value = React.useMemo(
    () => ({
      triggerRef,
      open,
      onOpenChange,
      disableRecentsDialogOpen,
      setDisableRecentsDialogOpen,
    }),
    [disableRecentsDialogOpen, onOpenChange, open, triggerRef],
  );

  return <HeaderMenuContext.Provider value={value}>{children}</HeaderMenuContext.Provider>;
}
