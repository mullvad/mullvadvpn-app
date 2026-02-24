import React from 'react';

import { MenuOption } from '../menu-option';
import { MenuDivider, MenuPopup, MenuTitle } from './components';
import { useEffectSetTriggerAttributes } from './hooks';
import { MenuProvider, useMenuContext } from './MenuContext';

export type MenuProps = React.PropsWithChildren<{
  triggerRef: React.RefObject<HTMLButtonElement | null>;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}>;

function MenuImpl({ children }: Omit<MenuProps, 'triggerRef' | 'open' | 'onOpenChange'>) {
  useEffectSetTriggerAttributes();
  const { open } = useMenuContext();

  if (!open) {
    return null;
  }

  return <>{children}</>;
}

function Menu({ triggerRef, open, onOpenChange, ...props }: MenuProps) {
  return (
    <MenuProvider triggerRef={triggerRef} open={open} onOpenChange={onOpenChange}>
      <MenuImpl {...props} />
    </MenuProvider>
  );
}

const MenuNamespace = Object.assign(Menu, {
  Popup: MenuPopup,
  Option: MenuOption,
  Divider: MenuDivider,
  Title: MenuTitle,
});

export { MenuNamespace as Menu };
