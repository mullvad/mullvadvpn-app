import { messages } from '../../../../../../../../shared/gettext';
import { FlexRow } from '../../../../../../../lib/components/flex-row';
import { ListItem } from '../../../../../../../lib/components/list-item';
import { BlockAdsSwitch } from './BlockAdsSwitch';

export function BlockAdsSetting() {
  return (
    <ListItem level={1}>
      <ListItem.Item>
        <ListItem.Content>
          <BlockAdsSwitch>
            <FlexRow $padding={{ left: 'medium' }}>
              <BlockAdsSwitch.Label variant="bodySmall">
                {
                  // TRANSLATORS: Label for settings that enables ad blocking.
                  messages.pgettext('vpn-settings-view', 'Ads')
                }
              </BlockAdsSwitch.Label>
            </FlexRow>
            <BlockAdsSwitch.Trigger>
              <BlockAdsSwitch.Thumb />
            </BlockAdsSwitch.Trigger>
          </BlockAdsSwitch>
        </ListItem.Content>
      </ListItem.Item>
    </ListItem>
  );
}
