import { DownloadProgress, Label } from './components';
import { useShowDownloadProgress } from './hooks';

export function DownloadDetails() {
  const showDownloadProgress = useShowDownloadProgress();

  return (
    <>
      <Label />
      {showDownloadProgress ? <DownloadProgress /> : null}
    </>
  );
}
