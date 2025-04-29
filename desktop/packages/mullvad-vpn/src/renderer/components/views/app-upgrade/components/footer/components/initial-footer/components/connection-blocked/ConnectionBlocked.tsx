import { messages } from '../../../../../../../../../../shared/gettext';
import { Flex, LabelTiny } from '../../../../../../../../../lib/components';
import { Dot } from '../../../../../../../../../lib/components/dot';
import { DownloadProgress } from '../../../../../download-progress';
import { LaunchInstallerButton } from '../../../../../launch-installer-button';

export function ConnectionBlocked() {
  return (
    <Flex $padding="large" $flexDirection="column">
      <Flex $gap="medium" $flexDirection="column" $margin={{ bottom: 'medium' }}>
        <Flex $gap="tiny" $alignItems="center">
          <Dot size="small" variant="error" />
          <LabelTiny>
            {
              // TRANSLATORS: Label displayed when an error occurred due to the connection being blocked
              messages.pgettext(
                'app-upgrade-view',
                'Connection blocked. Try changing server or other settings',
              )
            }
          </LabelTiny>
        </Flex>
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
