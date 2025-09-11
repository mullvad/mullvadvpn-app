import { messages } from '../../../../../../../../shared/gettext';
import { FlexRow } from '../../../../../../../lib/components/flex-row';
import { ToggleListItem } from '../../../../../../toggle-list-item';
import { useDns } from '../../hooks';

export function BlockGamblingSetting() {
  const [dns, setBlockGambling] = useDns('blockGambling');

  return (
    <ToggleListItem
      level={1}
      animation={undefined}
      disabled={dns.state === 'custom'}
      checked={dns.state === 'default' && dns.defaultOptions.blockGambling}
      onCheckedChange={setBlockGambling}>
      <FlexRow $padding={{ left: 'medium' }}>
        <ToggleListItem.Label variant="bodySmall">
          {
            // TRANSLATORS: Label for settings that enables block of gamling related websites.
            messages.pgettext('vpn-settings-view', 'Gambling')
          }
        </ToggleListItem.Label>
      </FlexRow>
      <ToggleListItem.Switch />
    </ToggleListItem>
  );
}
