import { messages } from '../../../../../../../../../../shared/gettext';
import { Flex } from '../../../../../../../../../lib/components';
import { ConnectionBlockedLabel } from '../../../../../connection-blocked-label';
import { DownloadProgress } from '../../../../../download-progress';
import { ResumeButton } from '../../../../../resume-button';

export function ConnectionBlocked() {
  return (
    <Flex padding="large" flexDirection="column">
      <Flex gap="medium" flexDirection="column" margin={{ bottom: 'medium' }}>
        <ConnectionBlockedLabel />
        <DownloadProgress />
      </Flex>
      <Flex flexDirection="column">
        <ResumeButton disabled>
          {
            // TRANSLATORS: Button text to resume updating
            messages.pgettext('app-upgrade-view', 'Resume')
          }
        </ResumeButton>
      </Flex>
    </Flex>
  );
}
