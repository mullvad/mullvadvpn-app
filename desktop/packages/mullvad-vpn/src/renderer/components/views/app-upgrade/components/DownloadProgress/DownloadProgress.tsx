import { Progress } from '../../../../../lib/components/progress';
import { useText, useValue } from './hooks';

export function DownloadProgress() {
  const value = useValue();
  const text = useText();

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
