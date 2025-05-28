import { messages } from '../../../../../../../../../../shared/gettext';
import { Flex } from '../../../../../../../../../lib/components';
import { UpgradeButton } from '../../../../../upgrade-button';

export function StartUpgrade() {
  return (
    <Flex $padding="large" $flexDirection="column">
      <Flex $flexDirection="column">
        <UpgradeButton>
          {
            // TRANSLATORS: Button text to download and install an update
            messages.pgettext('app-upgrade-view', 'Download & install')
          }
        </UpgradeButton>
      </Flex>
    </Flex>
  );
}
