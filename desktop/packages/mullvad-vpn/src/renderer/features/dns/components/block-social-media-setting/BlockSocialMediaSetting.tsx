import { messages } from '../../../../../shared/gettext';
import { ListItem } from '../../../../lib/components/list-item';
import { BlockSocialMediaSwitch } from '../block-social-media-switch/BlockSocialMediaSwitch';

export function BlockSocialMediaSetting() {
  return (
    <ListItem level={1}>
      <ListItem.Item>
        <ListItem.Content>
          <BlockSocialMediaSwitch>
            <BlockSocialMediaSwitch.Label variant="bodySmall">
              {
                // TRANSLATORS: Label for settings that enables block of social media.
                messages.pgettext('vpn-settings-view', 'Social media')
              }
            </BlockSocialMediaSwitch.Label>
            <BlockSocialMediaSwitch.Trigger>
              <BlockSocialMediaSwitch.Thumb />
            </BlockSocialMediaSwitch.Trigger>
          </BlockSocialMediaSwitch>
        </ListItem.Content>
      </ListItem.Item>
    </ListItem>
  );
}
