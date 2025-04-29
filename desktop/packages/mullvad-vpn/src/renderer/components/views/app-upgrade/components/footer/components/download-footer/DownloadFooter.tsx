import { Flex } from '../../../../../../../lib/components';
import { DownloadProgress } from '../../../download-progress';
import {
  ConnectionBlockedLabel,
  DownloadLabel,
  PauseDownloadButton,
  ResumeDownloadButton,
} from './components';
import { useShowConnectionBlockedLabel, useShowResumeDownloadButton } from './hooks';

export function DownloadFooter() {
  const showConnectionBlockedLabel = useShowConnectionBlockedLabel();
  const showResumeDownloadButton = useShowResumeDownloadButton();

  return (
    <Flex $padding="large" $flexDirection="column">
      <Flex $gap="medium" $flexDirection="column" $margin={{ bottom: 'medium' }}>
        {showConnectionBlockedLabel ? <ConnectionBlockedLabel /> : <DownloadLabel />}
        <DownloadProgress />
      </Flex>
      <Flex $flexDirection="column">
        {showResumeDownloadButton ? <ResumeDownloadButton /> : <PauseDownloadButton />}
      </Flex>
    </Flex>
  );
}
