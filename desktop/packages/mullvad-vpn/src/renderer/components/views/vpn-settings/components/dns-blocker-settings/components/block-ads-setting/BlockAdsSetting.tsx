import { messages } from '../../../../../../../../shared/gettext';
import { FlexRow } from '../../../../../../../lib/components/flex-row';
import { SettingsToggleListItem } from '../../../../../../settings-toggle-list-item';
import { useDns } from '../../hooks';

export function BlockAdsSetting() {
  const [dns, setBlockAds] = useDns('blockAds');

  return (
    <SettingsToggleListItem
      level={1}
      animation={undefined}
      disabled={dns.state === 'custom'}
      checked={dns.state === 'default' && dns.defaultOptions.blockAds}
      onCheckedChange={setBlockAds}>
      <FlexRow $padding={{ left: 'medium' }}>
        <SettingsToggleListItem.Label variant="bodySmall">
          {
            // TRANSLATORS: Label for settings that enables ad blocking.
            messages.pgettext('vpn-settings-view', 'Ads')
          }
        </SettingsToggleListItem.Label>
      </FlexRow>
      <SettingsToggleListItem.Switch />
    </SettingsToggleListItem>
  );
}
