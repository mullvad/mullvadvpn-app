import { messages } from '../../../../../../shared/gettext';
import { usePushChangelog } from '../../../../../history/hooks';
import { Icon } from '../../../../../lib/components';
import { ListItem, ListItemProps } from '../../../../../lib/components/list-item';

export type ChangelogListItemProps = Omit<ListItemProps, 'children'>;

export function ChangelogListItem(props: ChangelogListItemProps) {
  const pushChangelog = usePushChangelog();

  return (
    <ListItem {...props}>
      <ListItem.Trigger onClick={pushChangelog}>
        <ListItem.Item>
          <ListItem.Content>
            <ListItem.Label>
              {
                // TRANSLATORS: Label for changelog list item.
                messages.pgettext('app-info-view', 'What’s new')
              }
            </ListItem.Label>
            <Icon icon="chevron-right" />
          </ListItem.Content>
        </ListItem.Item>
      </ListItem.Trigger>
    </ListItem>
  );
}
