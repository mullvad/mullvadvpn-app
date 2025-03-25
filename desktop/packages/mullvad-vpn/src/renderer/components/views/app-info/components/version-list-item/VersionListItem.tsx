import { messages } from '../../../../../../shared/gettext';
import { Box, Icon } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';
import { Colors } from '../../../../../lib/foundations';
import { useVersionConsistent, useVersionCurrent } from '../../hooks';

export function VersionListItem() {
  const currentVersion = useVersionCurrent();
  const consistentVersion = useVersionConsistent();

  return (
    <ListItem>
      <ListItem.Item>
        <ListItem.Content>
          <ListItem.Group>
            {!consistentVersion && <Icon icon="alert-circle" color={Colors.red} />}
            <ListItem.Label>{messages.pgettext('app-info-view', 'Version')}</ListItem.Label>
          </ListItem.Group>
          <Box $width="24px" $height="24px" center>
            <ListItem.Text>{currentVersion}</ListItem.Text>
          </Box>
        </ListItem.Content>
      </ListItem.Item>
      {!consistentVersion && (
        <ListItem.Footer>
          <ListItem.Text>
            {messages.pgettext('app-info-view', 'App is out of sync. Please quit and restart.')}
          </ListItem.Text>
        </ListItem.Footer>
      )}
    </ListItem>
  );
}
