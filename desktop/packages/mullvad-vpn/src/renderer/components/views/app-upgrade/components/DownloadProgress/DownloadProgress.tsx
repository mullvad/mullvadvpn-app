import { Progress } from '../../../../../lib/components/progress';
import { useText, useValue } from './hooks';

export function DownloadProgress() {
  const text = useText();
  const value = useValue();

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
