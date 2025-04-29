import { messages } from '../../../../../../../../../../shared/gettext';
import { Flex } from '../../../../../../../../../lib/components';
import { ConnectionBlockedLabel } from '../../../../../connection-blocked-label';
import { DownloadProgress } from '../../../../../download-progress';
import { LaunchInstallerButton } from '../../../../../launch-installer-button';

export function ConnectionBlocked() {
  return (
    <Flex $padding="large" $flexDirection="column">
      <Flex $gap="medium" $flexDirection="column" $margin={{ bottom: 'medium' }}>
        <ConnectionBlockedLabel />
        <DownloadProgress />
      </Flex>
      <Flex $flexDirection="column">
        <LaunchInstallerButton disabled>
          {
            // TRANSLATORS: Button text to install an update
            messages.pgettext('app-upgrade-view', 'Install update')
          }
        </LaunchInstallerButton>
      </Flex>
    </Flex>
  );
}
