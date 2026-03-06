import {
  MenuOptionItem,
  MenuOptionItemIcon,
  MenuOptionItemLabel,
  MenuOptionTrigger,
} from './components';
import { MenuOptionProvider } from './MenuOptionContext';

export type MenuOptionProps = React.PropsWithChildren<{
  disabled?: boolean;
}>;

function MenuOption({ disabled, children }: MenuOptionProps) {
  return <MenuOptionProvider disabled={disabled}>{children}</MenuOptionProvider>;
}

const MenuOptionNamespace = Object.assign(MenuOption, {
  Item: MenuOptionItem,
  ItemLabel: MenuOptionItemLabel,
  ItemIcon: MenuOptionItemIcon,
  Trigger: MenuOptionTrigger,
});

export { MenuOptionNamespace as MenuOption };
