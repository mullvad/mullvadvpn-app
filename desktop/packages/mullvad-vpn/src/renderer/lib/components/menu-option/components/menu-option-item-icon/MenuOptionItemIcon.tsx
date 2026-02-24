import { Icon, type IconProps } from '../../../icon';
import { useMenuOptionContext } from '../../MenuOptionContext';

export type MenuOptionItemIconProps = IconProps;

export function MenuOptionItemIcon(props: MenuOptionItemIconProps) {
  const { disabled } = useMenuOptionContext();

  return <Icon size="small" color={disabled ? 'whiteAlpha20' : 'white'} {...props} />;
}
