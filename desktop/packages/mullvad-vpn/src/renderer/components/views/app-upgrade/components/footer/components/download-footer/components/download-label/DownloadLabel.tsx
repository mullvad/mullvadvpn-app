import { LabelTiny } from '../../../../../../../../../lib/components';
import { useMessage } from './hooks';

export function DownloadLabel() {
  const message = useMessage();

  return <LabelTiny>{message}</LabelTiny>;
}
