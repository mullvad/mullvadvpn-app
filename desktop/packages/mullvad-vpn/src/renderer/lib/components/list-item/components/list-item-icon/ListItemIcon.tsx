import { Icon, IconProps } from '../../../icon';
import { useListItem } from '../../ListItemContext';

export type ListItemIconProps = Omit<IconProps, 'size'>;

export function ListItemIcon({ ...props }: ListItemIconProps) {
  const { disabled } = useListItem();
  return <Icon aria-hidden="true" color={disabled ? 'whiteAlpha40' : 'white'} {...props} />;
}
