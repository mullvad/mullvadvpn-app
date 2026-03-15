import { ScrollToAnchorId } from '../../../shared/ipc-types';
import { useScrollToListItem } from '../../hooks';
import { ListItem, ListItemProps } from '../../lib/components/list-item';

export type SettingsListItemProps = ListItemProps & {
  anchorId?: ScrollToAnchorId;
  labelId?: string;
};

function SettingsListItem({ labelId, anchorId, ...props }: SettingsListItemProps) {
  const { ref, animation } = useScrollToListItem(anchorId);

  return (
    <ListItem ref={ref} aria-labelledby={labelId} tabIndex={-1} animation={animation} {...props} />
  );
}

const SettingsListItemNamespace = Object.assign(SettingsListItem, {
  Trigger: ListItem.Trigger,
  Item: ListItem.Item,
  Footer: ListItem.Footer,
});

export { SettingsListItemNamespace as SettingsListItem };
