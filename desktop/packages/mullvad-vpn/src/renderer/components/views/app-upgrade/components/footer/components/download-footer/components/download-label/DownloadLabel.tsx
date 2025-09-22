import { LabelTinySemiBold } from '../../../../../../../../../lib/components';
import { useMessage } from './hooks';

export function DownloadLabel() {
  const message = useMessage();

  return <LabelTinySemiBold>{message}</LabelTinySemiBold>;
}
