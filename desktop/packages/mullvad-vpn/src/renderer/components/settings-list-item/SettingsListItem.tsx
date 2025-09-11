import { ScrollToAnchorId } from '../../../shared/ipc-types';
import { useScrollToListItem } from '../../hooks';
import { ListItem, ListItemProps } from '../../lib/components/list-item';

export type SettingsListItemProps = ListItemProps & {
  anchorId?: ScrollToAnchorId;
};

function SettingsListItem({ anchorId, ...props }: SettingsListItemProps) {
  const { ref, animation } = useScrollToListItem(anchorId);

  return <ListItem ref={ref} animation={animation} {...props} />;
}

const SettingsListItemNamespace = Object.assign(SettingsListItem, {
  Content: ListItem.Content,
  Label: ListItem.Label,
  Group: ListItem.Group,
  Text: ListItem.Text,
  Trigger: ListItem.Trigger,
  Item: ListItem.Item,
  Footer: ListItem.Footer,
  Icon: ListItem.Icon,
  TextField: ListItem.TextField,
});

export { SettingsListItemNamespace as SettingsListItem };
