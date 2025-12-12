import { messages } from '../../../../../../../../../../shared/gettext';
import { Flex, Icon, LabelTinySemiBold } from '../../../../../../../../../lib/components';
import { DownloadProgress } from '../../../../../download-progress';
import { LaunchInstallerButton } from '../../../../../launch-installer-button';

export function InstallerReady() {
  return (
    <Flex padding="large" flexDirection="column">
      <Flex gap="medium" flexDirection="column" margin={{ bottom: 'medium' }}>
        <Flex gap="tiny" alignItems="center">
          <Icon icon="checkmark" color="green" size="small" />
          <LabelTinySemiBold>
            {
              // TRANSLATORS: Label displayed above a progress bar when the update is verified successfully
              messages.pgettext('app-upgrade-view', 'Verification successful!')
            }
          </LabelTinySemiBold>
        </Flex>
        <DownloadProgress />
      </Flex>
      <Flex flexDirection="column">
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
