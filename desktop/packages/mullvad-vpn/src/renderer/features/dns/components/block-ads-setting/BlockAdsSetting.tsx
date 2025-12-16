import { messages } from '../../../../../shared/gettext';
import { ListItem } from '../../../../lib/components/list-item';
import { BlockAdsSwitch } from '../block-ads-switch/BlockAdsSwitch';

export function BlockAdsSetting() {
  return (
    <ListItem level={1}>
      <ListItem.Item>
        <ListItem.Content>
          <BlockAdsSwitch>
            <BlockAdsSwitch.Label variant="bodySmall">
              {
                // TRANSLATORS: Label for settings that enables ad blocking.
                messages.pgettext('vpn-settings-view', 'Ads')
              }
            </BlockAdsSwitch.Label>
            <BlockAdsSwitch.Trigger>
              <BlockAdsSwitch.Thumb />
            </BlockAdsSwitch.Trigger>
          </BlockAdsSwitch>
        </ListItem.Content>
      </ListItem.Item>
    </ListItem>
  );
}
