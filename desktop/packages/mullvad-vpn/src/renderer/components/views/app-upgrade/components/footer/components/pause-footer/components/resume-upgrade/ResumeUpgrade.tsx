import { messages } from '../../../../../../../../../../shared/gettext';
import { Flex, LabelTiny } from '../../../../../../../../../lib/components';
import { DownloadProgress } from '../../../../../download-progress';
import { ResumeButton } from '../../../../../resume-button';

export function ResumeUpgrade() {
  return (
    <Flex $padding="large" $flexDirection="column">
      <Flex $gap="medium" $flexDirection="column" $margin={{ bottom: 'medium' }}>
        <LabelTiny>
          {
            // TRANSLATORS: Label displayed above a progress bar when the update is verified successfully
            messages.pgettext('app-upgrade-view', 'Download paused')
          }
        </LabelTiny>
        <DownloadProgress />
      </Flex>
      <Flex $flexDirection="column">
        <ResumeButton>
          {
            // TRANSLATORS: Button text to resume updating
            messages.pgettext('app-upgrade-view', 'Resume')
          }
        </ResumeButton>
      </Flex>
    </Flex>
  );
}
