import { messages } from '../../../../../shared/gettext';
import { FlexRow } from '../../../../lib/components/flex-row';
import { ListItem } from '../../../../lib/components/list-item';
import { BlockSocialMediaSwitch } from '../block-social-media-switch/BlockSocialMediaSwitch';

export function BlockSocialMediaSetting() {
  return (
    <ListItem level={1}>
      <ListItem.Item>
        <ListItem.Content>
          <BlockSocialMediaSwitch>
            <FlexRow padding={{ left: 'medium' }}>
              <BlockSocialMediaSwitch.Label variant="bodySmall">
                {
                  // TRANSLATORS: Label for settings that enables block of social media.
                  messages.pgettext('vpn-settings-view', 'Social media')
                }
              </BlockSocialMediaSwitch.Label>
            </FlexRow>
            <BlockSocialMediaSwitch.Trigger>
              <BlockSocialMediaSwitch.Thumb />
            </BlockSocialMediaSwitch.Trigger>
          </BlockSocialMediaSwitch>
        </ListItem.Content>
      </ListItem.Item>
    </ListItem>
  );
}
