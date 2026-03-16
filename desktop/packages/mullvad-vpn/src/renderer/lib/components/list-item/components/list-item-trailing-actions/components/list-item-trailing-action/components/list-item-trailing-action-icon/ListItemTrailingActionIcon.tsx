import { Icon, type IconProps } from '../../../../../../../icon';
import { useListItemContext } from '../../../../../../ListItemContext';

export type ListItemTrailingActionIconProps = Omit<IconProps, 'size'>;

export function ListItemTrailingActionIcon(props: ListItemTrailingActionIconProps) {
  const { disabled } = useListItemContext();

  return <Icon aria-hidden="true" color={disabled ? 'whiteAlpha40' : 'white'} {...props} />;
}
