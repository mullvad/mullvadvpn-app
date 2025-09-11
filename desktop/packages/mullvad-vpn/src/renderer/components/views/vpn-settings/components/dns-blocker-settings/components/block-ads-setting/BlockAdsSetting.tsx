import { messages } from '../../../../../../../../shared/gettext';
import { FlexRow } from '../../../../../../../lib/components/flex-row';
import { ToggleListItem } from '../../../../../../toggle-list-item';
import { useDns } from '../../hooks';

export function BlockAdsSetting() {
  const [dns, setBlockAds] = useDns('blockAds');

  return (
    <ToggleListItem
      level={1}
      animation={undefined}
      disabled={dns.state === 'custom'}
      checked={dns.state === 'default' && dns.defaultOptions.blockAds}
      onCheckedChange={setBlockAds}>
      <FlexRow $padding={{ left: 'medium' }}>
        <ToggleListItem.Label variant="bodySmall">
          {
            // TRANSLATORS: Label for settings that enables ad blocking.
            messages.pgettext('vpn-settings-view', 'Ads')
          }
        </ToggleListItem.Label>
      </FlexRow>
      <ToggleListItem.Switch />
    </ToggleListItem>
  );
}
