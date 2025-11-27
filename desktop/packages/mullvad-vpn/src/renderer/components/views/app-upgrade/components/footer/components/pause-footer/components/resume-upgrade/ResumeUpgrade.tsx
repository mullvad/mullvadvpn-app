import { messages } from '../../../../../../../../../../shared/gettext';
import { Flex, LabelTinySemiBold } from '../../../../../../../../../lib/components';
import { DownloadProgress } from '../../../../../download-progress';
import { ResumeButton } from '../../../../../resume-button';

export function ResumeUpgrade() {
  return (
    <Flex padding="large" flexDirection="column">
      <Flex gap="medium" flexDirection="column" margin={{ bottom: 'medium' }}>
        <LabelTinySemiBold>
          {
            // TRANSLATORS: Label displayed above a progress bar when the update is verified successfully
            messages.pgettext('app-upgrade-view', 'Download paused')
          }
        </LabelTinySemiBold>
        <DownloadProgress />
      </Flex>
      <Flex flexDirection="column">
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
