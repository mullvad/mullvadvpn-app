import { Flex, Icon, LabelTiny } from '../../../../../../../lib/components';
import { Colors } from '../../../../../../../lib/foundations';
import { useMessage } from './hooks';

export function UpgradeError() {
  const message = useMessage();

  return (
    <Flex $gap="tiny" $flexDirection="row">
      <div>
        <Icon size="small" icon="alert-circle" color={Colors.red} />
      </div>
      <Flex $flexDirection="column">
        <LabelTiny>{message}</LabelTiny>
      </Flex>
    </Flex>
  );
}
