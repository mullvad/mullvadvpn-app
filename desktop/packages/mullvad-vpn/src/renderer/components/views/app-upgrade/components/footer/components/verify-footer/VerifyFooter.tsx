import { messages } from '../../../../../../../../shared/gettext';
import { Flex, LabelTinySemiBold, Spinner } from '../../../../../../../lib/components';
import { DownloadProgress } from '../../../download-progress';
import { PauseButton } from '../../../pause-button';

export function VerifyFooter() {
  return (
    <Flex padding="large" flexDirection="column">
      <Flex gap="medium" flexDirection="column" margin={{ bottom: 'medium' }}>
        <Flex gap="tiny" alignItems="center">
          <Spinner size="small" />
          <LabelTinySemiBold>
            {
              // TRANSLATORS: Label displayed above a progress bar when the update is being verified
              messages.pgettext('app-upgrade-view', 'Verifying installer...')
            }
          </LabelTinySemiBold>
        </Flex>
        <DownloadProgress />
      </Flex>
      <Flex flexDirection="column">
        <PauseButton disabled>
          {
            // TRANSLATORS: Button text to pause the download of an update
            messages.pgettext('app-upgrade-view', 'Pause')
          }
        </PauseButton>
      </Flex>
    </Flex>
  );
}
