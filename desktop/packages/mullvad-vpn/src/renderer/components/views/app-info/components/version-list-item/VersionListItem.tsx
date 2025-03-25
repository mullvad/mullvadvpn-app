import { messages } from '../../../../../../shared/gettext';
import { Icon } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';
import { Colors } from '../../../../../lib/foundations';
import { useVersionCurrent } from '../../hooks';
import { useShowAlert, useShowFooter } from './hooks';

export function VersionListItem() {
  const { current } = useVersionCurrent();
  const showAlert = useShowAlert();
  const showFooter = useShowFooter();

  return (
    <ListItem>
      <ListItem.Item>
        <ListItem.Content>
          <ListItem.Group>
            {showAlert && <Icon icon="alert-circle" color={Colors.red} />}
            <ListItem.Label>
              {
                // TRANSLATORS: Label for version list item.
                messages.pgettext('app-info-view', 'Version')
              }
            </ListItem.Label>
          </ListItem.Group>
          <ListItem.Text>{current}</ListItem.Text>
        </ListItem.Content>
      </ListItem.Item>
      {showFooter && (
        <ListItem.Footer>
          <ListItem.Text>
            {
              // TRANSLATORS: Description for version list item when app is out of sync.
              messages.pgettext('app-info-view', 'App is out of sync. Please quit and restart.')
            }
          </ListItem.Text>
        </ListItem.Footer>
      )}
    </ListItem>
  );
}
