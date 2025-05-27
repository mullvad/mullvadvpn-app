import { Flex, Icon, LabelTiny } from '../../../../../../../lib/components';
import { Colors } from '../../../../../../../lib/foundations';
import { DownloadProgress } from '../../../download-progress';
import { ManualDownloadLink, ReportProblemButton, RetryButton } from './components';
import { useMessage, useShowManualDownloadLink } from './hooks';

export function ErrorFooter() {
  const message = useMessage();
  const showManualDownloadLink = useShowManualDownloadLink();

  return (
    <Flex $padding="large" $flexDirection="column">
      <Flex $gap="medium" $flexDirection="column" $margin={{ bottom: 'medium' }}>
        <Flex $gap="tiny" $flexDirection="row">
          <div>
            <Icon size="small" icon="alert-circle" color={Colors.red} />
          </div>
          <Flex $flexDirection="column">
            <LabelTiny>{message}</LabelTiny>
          </Flex>
        </Flex>
        <DownloadProgress />
      </Flex>
      <Flex $gap="medium" $flexDirection="column">
        {showManualDownloadLink && <ManualDownloadLink />}
        <ReportProblemButton />
        <RetryButton />
      </Flex>
    </Flex>
  );
}
