import { messages } from '../../../../../../../../../../shared/gettext';
import { Flex } from '../../../../../../../../../lib/components';
import { UpgradeButton } from '../../../../../upgrade-button';
import { useDisabled } from './hooks';

export function StartUpgrade() {
  const disabled = useDisabled();

  return (
    <Flex padding="large" flexDirection="column">
      <Flex flexDirection="column">
        <UpgradeButton disabled={disabled}>
          {
            // TRANSLATORS: Button text to download and install an update
            messages.pgettext('app-upgrade-view', 'Download & install')
          }
        </UpgradeButton>
      </Flex>
    </Flex>
  );
}
