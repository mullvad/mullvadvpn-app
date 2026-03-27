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
          <ListItem.Item.Label>
            {
              // TRANSLATORS: Label for changelog list item.
              messages.pgettext('app-info-view', 'What’s new')
            }
          </ListItem.Item.Label>
          <ListItem.Item.ActionGroup>
            <Icon icon="chevron-right" />
          </ListItem.Item.ActionGroup>
        </ListItem.Item>
      </ListItem.Trigger>
    </ListItem>
  );
}
