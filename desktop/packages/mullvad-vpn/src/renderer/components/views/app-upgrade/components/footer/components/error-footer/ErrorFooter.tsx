import { Flex, Icon, LabelTiny } from '../../../../../../../lib/components';
import { Colors } from '../../../../../../../lib/foundations';
import { DownloadProgress } from '../../../download-progress';
import { ManualDownloadButton, ReportProblemButton, RetryButton } from './components';
import { useMessage, useShowManualDownloadButton } from './hooks';

export function ErrorFooter() {
  const message = useMessage();
  const showManualDownloadButton = useShowManualDownloadButton();

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
        <ReportProblemButton />
        {showManualDownloadButton ? <ManualDownloadButton /> : <RetryButton />}
      </Flex>
    </Flex>
  );
}
