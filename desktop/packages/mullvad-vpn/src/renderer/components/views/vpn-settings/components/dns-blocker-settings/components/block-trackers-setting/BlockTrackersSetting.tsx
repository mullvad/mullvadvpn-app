import { messages } from '../../../../../../../../shared/gettext';
import { FlexRow } from '../../../../../../../lib/components/flex-row';
import { ToggleListItem } from '../../../../../../toggle-list-item';
import { useDns } from '../../hooks';

export function BlockTrackersSetting() {
  const [dns, setBlockTrackers] = useDns('blockTrackers');

  return (
    <ToggleListItem
      level={1}
      animation={undefined}
      disabled={dns.state === 'custom'}
      checked={dns.state === 'default' && dns.defaultOptions.blockTrackers}
      onCheckedChange={setBlockTrackers}>
      <FlexRow $padding={{ left: 'medium' }}>
        <ToggleListItem.Label variant="bodySmall">
          {
            // TRANSLATORS: Label for settings that enables tracker blocking.
            messages.pgettext('vpn-settings-view', 'Trackers')
          }
        </ToggleListItem.Label>
      </FlexRow>
      <ToggleListItem.Switch />
    </ToggleListItem>
  );
}
