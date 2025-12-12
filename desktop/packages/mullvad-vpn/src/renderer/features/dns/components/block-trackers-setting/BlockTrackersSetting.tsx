import { messages } from '../../../../../shared/gettext';
import { FlexRow } from '../../../../lib/components/flex-row';
import { ListItem } from '../../../../lib/components/list-item';
import { BlockTrackersSwitch } from '../block-trackers-switch/BlockTrackersSwitch';

export function BlockTrackersSetting() {
  return (
    <ListItem level={1}>
      <ListItem.Item>
        <ListItem.Content>
          <BlockTrackersSwitch>
            <FlexRow padding={{ left: 'medium' }}>
              <BlockTrackersSwitch.Label variant="bodySmall">
                {
                  // TRANSLATORS: Label for settings that enables tracker blocking.
                  messages.pgettext('vpn-settings-view', 'Trackers')
                }
              </BlockTrackersSwitch.Label>
            </FlexRow>
            <BlockTrackersSwitch.Trigger>
              <BlockTrackersSwitch.Thumb />
            </BlockTrackersSwitch.Trigger>
          </BlockTrackersSwitch>
        </ListItem.Content>
      </ListItem.Item>
    </ListItem>
  );
}
