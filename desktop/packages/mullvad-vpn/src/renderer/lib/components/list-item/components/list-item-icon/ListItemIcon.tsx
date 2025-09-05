import { Icon, IconProps } from '../../../icon';
import { useListItemContext } from '../../ListItemContext';

export type ListItemIconProps = Omit<IconProps, 'size'>;

export function ListItemIcon({ ...props }: ListItemIconProps) {
  const { disabled } = useListItemContext();
  return <Icon aria-hidden="true" color={disabled ? 'whiteAlpha40' : 'white'} {...props} />;
}
