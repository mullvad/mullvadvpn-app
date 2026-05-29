import React from 'react';

import type { MenuProps } from './Menu';

type MenuContextProps = Omit<MenuProviderProps, 'children'> & {
  popupId: string;
};

const MenuContext = React.createContext<MenuContextProps | undefined>(undefined);

export const useMenuContext = (): MenuContextProps => {
  const context = React.useContext(MenuContext);
  if (!context) {
    throw new Error('useMenuContext must be used within a MenuProvider');
  }
  return context;
};

type MenuProviderProps = React.PropsWithChildren<{
  open: MenuProps['open'];
  onOpenChange?: MenuProps['onOpenChange'];
  triggerRef: MenuProps['triggerRef'];
}>;

export function MenuProvider({ children, ...props }: MenuProviderProps) {
  const popupId = React.useId();

  const value = React.useMemo(() => ({ popupId, ...props }), [props, popupId]);

  return <MenuContext.Provider value={value}>{children}</MenuContext.Provider>;
}
