import { Flex } from '../../../../../../../lib/components';
import { DownloadProgress, Label } from './components';
import { useShowDownloadProgress } from './hooks';

export function DownloadDetails() {
  const showDownloadProgress = useShowDownloadProgress();

  return (
    <Flex $gap="medium" $flexDirection="column">
      <Label />
      {showDownloadProgress ? <DownloadProgress /> : null}
    </Flex>
  );
}
