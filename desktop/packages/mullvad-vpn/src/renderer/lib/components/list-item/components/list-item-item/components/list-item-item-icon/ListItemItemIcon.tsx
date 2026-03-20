import { Icon, IconProps } from '../../../../../icon';
import { useListItemContext } from '../../../../ListItemContext';

export type ListItemItemIconProps = Omit<IconProps, 'size'>;

export function ListItemItemIcon({ ...props }: ListItemItemIconProps) {
  const { disabled } = useListItemContext();
  return <Icon aria-hidden="true" color={disabled ? 'whiteAlpha40' : 'white'} {...props} />;
}
