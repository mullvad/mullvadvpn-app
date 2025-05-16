import { messages } from '../../../../../../../../../../shared/gettext';
import { Flex, Icon, LabelTiny } from '../../../../../../../../../lib/components';
import { Colors } from '../../../../../../../../../lib/foundations';
import { DownloadProgress } from '../../../../../download-progress';
import { LaunchInstallerButton } from '../../../../../launch-installer-button';

export function InstallerReady() {
  return (
    <Flex $padding="large" $flexDirection="column">
      <Flex $gap="medium" $flexDirection="column" $margin={{ bottom: 'medium' }}>
        <Flex $gap="tiny" $alignItems="center">
          <Icon icon="checkmark" color={Colors.green} size="small" />
          <LabelTiny>
            {
              // TRANSLATORS: Label displayed above a progress bar when the update is verified successfully
              messages.pgettext('app-upgrade-view', 'Verification successful!')
            }
          </LabelTiny>
        </Flex>
        <DownloadProgress />
      </Flex>
      <Flex $flexDirection="column">
        <LaunchInstallerButton>
          {
            // TRANSLATORS: Button text to install an update
            messages.pgettext('app-upgrade-view', 'Install update')
          }
        </LaunchInstallerButton>
      </Flex>
    </Flex>
  );
}
