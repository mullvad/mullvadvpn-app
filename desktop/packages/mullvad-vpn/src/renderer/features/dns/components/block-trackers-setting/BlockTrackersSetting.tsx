import { messages } from '../../../../../shared/gettext';
import { ListItem } from '../../../../lib/components/list-item';
import { BlockTrackersSwitch } from '../block-trackers-switch/BlockTrackersSwitch';

export function BlockTrackersSetting() {
  return (
    <ListItem level={1}>
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
