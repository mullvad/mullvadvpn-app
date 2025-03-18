import { useAppUpgradeDownloadProgressValue } from '../../../../../hooks';
import { Progress } from '../../../../../lib/components/progress';
import { useText } from './hooks';

export function DownloadProgress() {
  const text = useText();
  const value = useAppUpgradeDownloadProgressValue();

  return (
    <Progress value={value}>
      <Progress.Track>
        <Progress.Range />
      </Progress.Track>
      <Progress.Footer>
        <Progress.Percent />
        <Progress.Text>{text}</Progress.Text>
      </Progress.Footer>
    </Progress>
  );
}
