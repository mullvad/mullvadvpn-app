import { messages } from '../../../../../../../../shared/gettext';
import { FlexRow } from '../../../../../../../lib/components/flex-row';
import { SettingsToggleListItem } from '../../../../../../settings-toggle-list-item';
import { useDns } from '../../hooks';

export function BlockTrackersSetting() {
  const [dns, setBlockTrackers] = useDns('blockTrackers');

  return (
    <SettingsToggleListItem
      level={1}
      animation={false}
      disabled={dns.state === 'custom'}
      checked={dns.state === 'default' && dns.defaultOptions.blockTrackers}
      onCheckedChange={setBlockTrackers}>
      <FlexRow $padding={{ left: 'medium' }}>
        <SettingsToggleListItem.Label variant="bodySmall">
          {
            // TRANSLATORS: Label for settings that enables tracker blocking.
            messages.pgettext('vpn-settings-view', 'Trackers')
          }
        </SettingsToggleListItem.Label>
      </FlexRow>
      <SettingsToggleListItem.Switch />
    </SettingsToggleListItem>
  );
}
