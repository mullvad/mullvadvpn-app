import { LabelTiny } from '../../../../../../../lib/components';
import { useMessage } from './hooks';

export function DownloadProgress() {
  const message = useMessage();

  return <LabelTiny>{message}</LabelTiny>;
}
