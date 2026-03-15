import { messages } from '../../../../../../shared/gettext';
import { Icon } from '../../../../../lib/components';
import { ListItem, ListItemProps } from '../../../../../lib/components/list-item';
import { useVersionCurrent } from '../../../../../redux/hooks';
import { useShowAlert, useShowFooter } from './hooks';

export type VersionListItemProps = Omit<ListItemProps, 'children'>;

export function VersionListItem(props: VersionListItemProps) {
  const { current } = useVersionCurrent();
  const showAlert = useShowAlert();
  const showFooter = useShowFooter();

  return (
    <ListItem {...props}>
      <ListItem.Item>
        <ListItem.Item.Group gap="small">
          {showAlert && <Icon icon="alert-circle" color="red" />}
          <ListItem.Item.Label>
            {
              // TRANSLATORS: Label for version list item.
              messages.pgettext('app-info-view', 'Version')
            }
          </ListItem.Item.Label>
        </ListItem.Item.Group>
        <ListItem.Item.ActionGroup>
          <ListItem.Item.Text>{current}</ListItem.Item.Text>
        </ListItem.Item.ActionGroup>
      </ListItem.Item>
      {showFooter && (
        <ListItem.Footer>
          <ListItem.Footer.Text>
            {
              // TRANSLATORS: Description for version list item when app is out of sync.
              messages.pgettext('app-info-view', 'App is out of sync. Please quit and restart.')
            }
          </ListItem.Footer.Text>
        </ListItem.Footer>
      )}
    </ListItem>
  );
}
