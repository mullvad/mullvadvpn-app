import { messages } from '../../../../../../../../shared/gettext';
import { FlexRow } from '../../../../../../../lib/components/flex-row';
import { ToggleListItem } from '../../../../../../toggle-list-item';
import { useDns } from '../../hooks';

export function BlockSocialMediaSetting() {
  const [dns, setBlockSocialMedia] = useDns('blockSocialMedia');

  return (
    <ToggleListItem
      level={1}
      disabled={dns.state === 'custom'}
      checked={dns.state === 'default' && dns.defaultOptions.blockSocialMedia}
      onCheckedChange={setBlockSocialMedia}>
      <FlexRow $padding={{ left: 'medium' }}>
        <ToggleListItem.Label variant="bodySmall">
          {
            // TRANSLATORS: Label for settings that enables block of social media.
            messages.pgettext('vpn-settings-view', 'Social media')
          }
        </ToggleListItem.Label>
      </FlexRow>
      <ToggleListItem.Switch />
    </ToggleListItem>
  );
}
