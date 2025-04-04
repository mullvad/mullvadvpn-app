import { useAppUpgradeDownloadProgressValue } from '../../../../../hooks';
import { Progress } from '../../../../../lib/components/progress';
import { useMessage } from './hooks';

export function DownloadProgress() {
  const message = useMessage();
  const value = useAppUpgradeDownloadProgressValue();

  return (
    <Progress value={value}>
      <Progress.Track>
        <Progress.Range />
      </Progress.Track>
      <Progress.Footer>
        <Progress.Percent />
        <Progress.Text>{message}</Progress.Text>
      </Progress.Footer>
    </Progress>
  );
}
