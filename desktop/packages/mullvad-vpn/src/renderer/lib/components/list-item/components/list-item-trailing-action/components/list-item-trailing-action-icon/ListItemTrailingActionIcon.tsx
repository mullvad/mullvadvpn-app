import { Icon, IconProps } from '../../../../../icon';
import { useListItemContext } from '../../../../ListItemContext';

type ListItemIconProps = Omit<IconProps, 'size'>;

export function ListItemTrailingActionIcon({ ...props }: ListItemIconProps) {
  const { disabled } = useListItemContext();
  return <Icon aria-hidden="true" color={disabled ? 'whiteAlpha40' : 'white'} {...props} />;
}
