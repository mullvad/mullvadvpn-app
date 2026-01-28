import { Icon, IconProps } from '../../../../../icon';
import { useListItemContext } from '../../../../ListItemContext';
import { useListItemTriggerContext } from '../../../list-item-trigger/ListItemTriggerContext';

type ListItemIconProps = Omit<IconProps, 'size'>;

export function ListItemTrailingActionIcon({ ...props }: ListItemIconProps) {
  const { disabled: listItemDisabled } = useListItemContext();
  const { disabled: listItemTriggerDisabled } = useListItemTriggerContext();

  const disabled = listItemTriggerDisabled ?? listItemDisabled;

  return <Icon aria-hidden="true" color={disabled ? 'whiteAlpha40' : 'white'} {...props} />;
}
