import { messages } from '../../../../../shared/gettext';
import { ListItem, ListItemProps } from '../../../../lib/components/list-item';
import { BlockTrackersSwitch } from '../block-trackers-switch/BlockTrackersSwitch';

export type BlockTrackersSettingProps = Omit<ListItemProps, 'children'>;

export function BlockTrackersSetting(props: BlockTrackersSettingProps) {
  return (
    <ListItem level={1} {...props}>
      <ListItem.Item>
        <ListItem.Content>
          <BlockTrackersSwitch>
            <BlockTrackersSwitch.Label variant="bodySmall">
              {
                // TRANSLATORS: Label for settings that enables tracker blocking.
                messages.pgettext('vpn-settings-view', 'Trackers')
              }
            </BlockTrackersSwitch.Label>
            <BlockTrackersSwitch.Trigger>
              <BlockTrackersSwitch.Thumb />
            </BlockTrackersSwitch.Trigger>
          </BlockTrackersSwitch>
        </ListItem.Content>
      </ListItem.Item>
    </ListItem>
  );
}
