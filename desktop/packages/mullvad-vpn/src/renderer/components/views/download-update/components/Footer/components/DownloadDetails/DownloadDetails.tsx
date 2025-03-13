import { AnimateHeight } from '../../../../../../AnimateHeight';
import { DownloadProgress, Label } from './components';
import { useShowDownloadProgress } from './hooks';

export function DownloadDetails() {
  const showDownloadProgress = useShowDownloadProgress();

  return (
    <AnimateHeight>
      <Label />
      {showDownloadProgress ? <DownloadProgress /> : null}
    </AnimateHeight>
  );
}
