import { messages } from '../../../../../shared/gettext';
import { ListItem } from '../../../../lib/components/list-item';
import { BlockAdultContentSwitch } from '../block-adult-content-switch/BlockAdultContentSwitch';

export function BlockAdultContentSetting() {
  return (
    <ListItem level={1}>
      <ListItem.Item>
        <ListItem.Content>
          <BlockAdultContentSwitch>
            <BlockAdultContentSwitch.Label variant="bodySmall">
              {
                // TRANSLATORS: Label for settings that enables block of adult content.
                messages.pgettext('vpn-settings-view', 'Adult content')
              }
            </BlockAdultContentSwitch.Label>
            <BlockAdultContentSwitch.Trigger>
              <BlockAdultContentSwitch.Thumb />
            </BlockAdultContentSwitch.Trigger>
          </BlockAdultContentSwitch>
        </ListItem.Content>
      </ListItem.Item>
    </ListItem>
  );
}
