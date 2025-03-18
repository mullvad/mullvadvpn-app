import { Flex, LabelTiny } from '../../../../../../../lib/components';
import { useTextServer } from './hooks';

export function DownloadProgress() {
  const textServer = useTextServer();

  return (
    <Flex $gap="small">
      <LabelTiny>{textServer}</LabelTiny>
    </Flex>
  );
}
