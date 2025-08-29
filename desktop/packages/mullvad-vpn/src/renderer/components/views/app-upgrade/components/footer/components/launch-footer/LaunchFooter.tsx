import { messages } from '../../../../../../../../shared/gettext';
import { Flex, Icon, LabelTiny } from '../../../../../../../lib/components';
import { DownloadProgress } from '../../../download-progress';
import { LaunchInstallerButton } from '../../../launch-installer-button';
import { useDisabled, useMessage } from './hooks';

export function LaunchFooter() {
  const disabled = useDisabled();
  const message = useMessage();

  return (
    <Flex $padding="large" $flexDirection="column">
      <Flex $gap="medium" $flexDirection="column" $margin={{ bottom: 'medium' }}>
        <Flex $gap="tiny" $alignItems="center">
          <Icon icon="checkmark" color="green" size="small" />
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
        <LaunchInstallerButton disabled={disabled}>{message}</LaunchInstallerButton>
      </Flex>
    </Flex>
  );
}
