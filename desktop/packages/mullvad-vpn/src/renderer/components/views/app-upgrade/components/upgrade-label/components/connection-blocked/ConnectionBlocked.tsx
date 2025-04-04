import { Flex, LabelTiny } from '../../../../../../../lib/components';
import { Dot } from '../../../../../../../lib/components/dot';
import { useMessage } from './hooks';

export function ConnectionBlocked() {
  const message = useMessage();

  return (
    <Flex $gap="small" $alignItems="baseline">
      <Dot size="small" variant="error" />
      <LabelTiny>{message}</LabelTiny>
    </Flex>
  );
}
