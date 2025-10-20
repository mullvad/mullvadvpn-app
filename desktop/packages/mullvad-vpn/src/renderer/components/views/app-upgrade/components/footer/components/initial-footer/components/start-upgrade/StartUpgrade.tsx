import { messages } from '../../../../../../../../../../shared/gettext';
import { Flex } from '../../../../../../../../../lib/components';
import { useHasUpgrade } from '../../../../../../hooks';
import { UpgradeButton } from '../../../../../upgrade-button';

export function StartUpgrade() {
  const hasUpgrade = useHasUpgrade();

  return (
    <Flex $padding="large" $flexDirection="column">
      <Flex $flexDirection="column">
        <UpgradeButton disabled={!hasUpgrade}>
          {
            // TRANSLATORS: Button text to download and install an update
            messages.pgettext('app-upgrade-view', 'Download & install')
          }
        </UpgradeButton>
      </Flex>
    </Flex>
  );
}
