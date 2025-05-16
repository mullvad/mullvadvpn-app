import { useAppUpgradeDownloadProgressValue } from '../../../../../hooks';
import { Progress } from '../../../../../lib/components/progress';
import { useDisabled, useMessage } from './hooks';

export function DownloadProgress() {
  const disabled = useDisabled();
  const message = useMessage();
  const value = useAppUpgradeDownloadProgressValue();

  return (
    <Progress value={value} disabled={disabled}>
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
