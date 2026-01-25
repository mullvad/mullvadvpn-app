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
        <ListItem.Group gap="small">
          {showAlert && <Icon icon="alert-circle" color="red" />}
          <ListItem.Label>
            {
              // TRANSLATORS: Label for version list item.
              messages.pgettext('app-info-view', 'Version')
            }
          </ListItem.Label>
        </ListItem.Group>
        <ListItem.ActionGroup>
          <ListItem.Text>{current}</ListItem.Text>
        </ListItem.ActionGroup>
      </ListItem.Item>
      {showFooter && (
        <ListItem.Footer>
          <ListItem.FooterText>
            {
              // TRANSLATORS: Description for version list item when app is out of sync.
              messages.pgettext('app-info-view', 'App is out of sync. Please quit and restart.')
            }
          </ListItem.FooterText>
        </ListItem.Footer>
      )}
    </ListItem>
  );
}
