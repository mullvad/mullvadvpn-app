import { messages } from '../../../../../../../../../../shared/gettext';
import { Flex } from '../../../../../../../../../lib/components';
import { ConnectionBlockedLabel } from '../../../../../connection-blocked-label';
import { UpgradeButton } from '../../../../../upgrade-button';

export function ConnectionBlocked() {
  return (
    <Flex padding="large" flexDirection="column">
      <Flex gap="medium" flexDirection="column" margin={{ bottom: 'medium' }}>
        <ConnectionBlockedLabel />
      </Flex>
      <Flex flexDirection="column">
        <UpgradeButton disabled>
          {
            // TRANSLATORS: Button text to download and install an update
            messages.pgettext('app-upgrade-view', 'Download & install')
          }
        </UpgradeButton>
      </Flex>
    </Flex>
  );
}
