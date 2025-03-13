import { AnimateHeight } from '../../../../../../AnimateHeight';
import { DownloadProgress, Label } from './components';

export function DownloadDetails() {
  return (
    <AnimateHeight>
      <Label />
      <DownloadProgress />
    </AnimateHeight>
  );
}
