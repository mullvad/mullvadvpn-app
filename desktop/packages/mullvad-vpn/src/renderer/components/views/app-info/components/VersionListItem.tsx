import { messages } from '../../../../../shared/gettext';
import { Box, Icon } from '../../../../lib/components';
import { ListItem } from '../../../../lib/components/list-item';
import { Colors } from '../../../../lib/foundations';
import { useSelector } from '../../../../redux/store';

export const VersionListItem = () => {
  const appVersion = useSelector((state) => state.version.current);
  const consistentVersion = useSelector((state) => state.version.consistent);

  return (
    <ListItem>
      <ListItem.Item>
        <ListItem.Content>
          <ListItem.Group>
            {!consistentVersion && <Icon icon="alert-circle" color={Colors.red} />}
            <ListItem.Label>
              {
                // TRANSLATORS: Label for version list item.
                messages.pgettext('app-info-view', 'Version')
              }
            </ListItem.Label>
          </ListItem.Group>
          <Box $width="24px" $height="24px" center>
            <ListItem.Text>{appVersion}</ListItem.Text>
          </Box>
        </ListItem.Content>
      </ListItem.Item>
      {!consistentVersion && (
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
};
