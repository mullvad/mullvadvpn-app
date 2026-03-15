import { ScrollToAnchorId } from '../../../shared/ipc-types';
import { useScrollToListItem } from '../../hooks';
import { ListItem, ListItemProps } from '../../lib/components/list-item';
import { SettingsListItemGroup } from './components';

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
  Label: ListItem.Item.Label,
  Group: SettingsListItemGroup,
  ActionGroup: ListItem.Item.ActionGroup,
  Text: ListItem.Item.Text,
  Trigger: ListItem.Trigger,
  Item: ListItem.Item,
  Footer: ListItem.Footer,
  FooterText: ListItem.Footer.Text,
  Icon: ListItem.Item.Icon,
  TextField: ListItem.Item.TextField,
});

export { SettingsListItemNamespace as SettingsListItem };
