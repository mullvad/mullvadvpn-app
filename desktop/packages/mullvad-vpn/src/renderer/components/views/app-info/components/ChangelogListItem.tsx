import { useCallback } from 'react';

import { messages } from '../../../../../shared/gettext';
import { Icon } from '../../../../lib/components';
import { ListItem } from '../../../../lib/components/list-item';
import { useHistory } from '../../../../lib/history';
import { RoutePath } from '../../../../lib/routes';

export function ChangelogListItem() {
  const history = useHistory();
  const navigate = useCallback(() => history.push(RoutePath.changelog), [history]);

  return (
    <ListItem>
      <ListItem.Item>
        <ListItem.Trigger onClick={navigate}>
          <ListItem.Content>
            <ListItem.Label>{messages.pgettext('settings-view', 'What’s new')}</ListItem.Label>
            <Icon icon="chevron-right" />
          </ListItem.Content>
        </ListItem.Trigger>
      </ListItem.Item>
    </ListItem>
  );
}
