import { messages } from '../../../../../../../../shared/gettext';
import { FlexRow } from '../../../../../../../lib/components/flex-row';
import { SettingsToggleListItem } from '../../../../../../settings-toggle-list-item';
import { useDns } from '../../hooks';

export function BlockAdultContentSetting() {
  const [dns, setBlockAdultContent] = useDns('blockAdultContent');

  return (
    <SettingsToggleListItem
      level={1}
      animation={false}
      disabled={dns.state === 'custom'}
      checked={dns.state === 'default' && dns.defaultOptions.blockAdultContent}
      onCheckedChange={setBlockAdultContent}>
      <FlexRow $padding={{ left: 'medium' }}>
        <SettingsToggleListItem.Label variant="bodySmall">
          {
            // TRANSLATORS: Label for settings that enables block of adult content.
            messages.pgettext('vpn-settings-view', 'Adult content')
          }
        </SettingsToggleListItem.Label>
      </FlexRow>
      <SettingsToggleListItem.Switch />
    </SettingsToggleListItem>
  );
}
