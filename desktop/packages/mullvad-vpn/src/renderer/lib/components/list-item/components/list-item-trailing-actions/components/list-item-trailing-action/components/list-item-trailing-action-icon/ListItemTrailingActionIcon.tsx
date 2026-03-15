import { Icon, type IconProps } from '../../../../../../../icon';
import { useListItemContext } from '../../../../../../ListItemContext';
import { useListItemTriggerContext } from '../../../../../list-item-trigger/ListItemTriggerContext';

export type ListItemTrailingActionIconProps = Omit<IconProps, 'size'>;

export function ListItemTrailingActionIcon(props: ListItemTrailingActionIconProps) {
  const { disabled: listItemDisabled } = useListItemContext();
  // TODO: Restructure this component to be a child of trigger to clarify
  // that it uses that context.
  const { disabled: listItemTriggerDisabled } = useListItemTriggerContext();

  const disabled = listItemTriggerDisabled ?? listItemDisabled;

  return <Icon aria-hidden="true" color={disabled ? 'whiteAlpha40' : 'white'} {...props} />;
}
