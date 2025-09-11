import { messages } from '../../../../../../../../shared/gettext';
import { FlexRow } from '../../../../../../../lib/components/flex-row';
import { SettingsToggleListItem } from '../../../../../../settings-toggle-list-item';
import { useDns } from '../../hooks';

export function BlockGamblingSetting() {
  const [dns, setBlockGambling] = useDns('blockGambling');

  return (
    <SettingsToggleListItem
      level={1}
      animation={undefined}
      disabled={dns.state === 'custom'}
      checked={dns.state === 'default' && dns.defaultOptions.blockGambling}
      onCheckedChange={setBlockGambling}>
      <FlexRow $padding={{ left: 'medium' }}>
        <SettingsToggleListItem.Label variant="bodySmall">
          {
            // TRANSLATORS: Label for settings that enables block of gamling related websites.
            messages.pgettext('vpn-settings-view', 'Gambling')
          }
        </SettingsToggleListItem.Label>
      </FlexRow>
      <SettingsToggleListItem.Switch />
    </SettingsToggleListItem>
  );
}
