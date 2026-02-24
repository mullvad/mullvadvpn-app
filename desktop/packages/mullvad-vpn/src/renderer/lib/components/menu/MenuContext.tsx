import React from 'react';

import type { MenuProps } from './Menu';

type MenuContextProps = Omit<MenuProviderProps, 'children'> & {
  popoverRef: React.RefObject<HTMLDivElement | null>;
  popoverId: string;
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
  triggerRef: MenuProps['triggerRef'];
  onOpenChange?: MenuProps['onOpenChange'];
}>;

export function MenuProvider({ children, ...props }: MenuProviderProps) {
  const popoverRef = React.useRef<HTMLDivElement>(null);
  const popoverId = React.useId();

  const value = React.useMemo(() => ({ popoverRef, popoverId, ...props }), [props, popoverId]);

  return <MenuContext.Provider value={value}>{children}</MenuContext.Provider>;
}
