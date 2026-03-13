import React from 'react';

type MenuOptionContextProps = Omit<MenuOptionProviderProps, 'children'>;

const MenuOptionContext = React.createContext<MenuOptionContextProps | undefined>(undefined);

export const useMenuOptionContext = (): MenuOptionContextProps => {
  const context = React.useContext(MenuOptionContext);
  if (!context) {
    throw new Error('useMenuOptionContext must be used within a MenuOptionProvider');
  }
  return context;
};

type MenuOptionProviderProps = React.PropsWithChildren<{
  disabled?: boolean;
}>;

export function MenuOptionProvider({ children, ...props }: MenuOptionProviderProps) {
  const value = React.useMemo(() => ({ ...props }), [props]);

  return <MenuOptionContext.Provider value={value}>{children}</MenuOptionContext.Provider>;
}
