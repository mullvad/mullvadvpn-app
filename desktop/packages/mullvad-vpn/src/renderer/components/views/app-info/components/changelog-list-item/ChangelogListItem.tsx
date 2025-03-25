import { messages } from '../../../../../../shared/gettext';
import { Icon } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';
import { usePushChangelog } from '../../hooks';

export function ChangelogListItem() {
  const pushChangelog = usePushChangelog();

  return (
    <ListItem>
      <ListItem.Item>
        <ListItem.Trigger onClick={pushChangelog}>
          <ListItem.Content>
            <ListItem.Label>
              {
                // TRANSLATORS: Label for changelog list item.
                messages.pgettext('app-info-view', 'What’s new')
              }
            </ListItem.Label>
            <Icon icon="chevron-right" />
          </ListItem.Content>
        </ListItem.Trigger>
      </ListItem.Item>
    </ListItem>
  );
}
