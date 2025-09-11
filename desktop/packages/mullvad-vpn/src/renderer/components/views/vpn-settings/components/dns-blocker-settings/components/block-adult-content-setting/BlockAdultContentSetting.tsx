import { messages } from '../../../../../../../../shared/gettext';
import { FlexRow } from '../../../../../../../lib/components/flex-row';
import { ToggleListItem } from '../../../../../../toggle-list-item';
import { useDns } from '../../hooks';

export function BlockAdultContentSetting() {
  const [dns, setBlockAdultContent] = useDns('blockAdultContent');

  return (
    <ToggleListItem
      level={1}
      animation={undefined}
      disabled={dns.state === 'custom'}
      checked={dns.state === 'default' && dns.defaultOptions.blockAdultContent}
      onCheckedChange={setBlockAdultContent}>
      <FlexRow $padding={{ left: 'medium' }}>
        <ToggleListItem.Label variant="bodySmall">
          {
            // TRANSLATORS: Label for settings that enables block of adult content.
            messages.pgettext('vpn-settings-view', 'Adult content')
          }
        </ToggleListItem.Label>
      </FlexRow>
      <ToggleListItem.Switch />
    </ToggleListItem>
  );
}
