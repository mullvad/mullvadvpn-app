import { Flex, Icon, LabelTiny } from '../../../../../../../lib/components';
import { Colors } from '../../../../../../../lib/foundations';
import { useText } from './hooks';

export function UpgradeError() {
  const text = useText();

  return (
    <Flex $gap="small" $flexDirection="row">
      <div>
        <Icon size="small" icon="alert-circle" color={Colors.red} />
      </div>
      <Flex $flexDirection="column">
        <LabelTiny>{text}</LabelTiny>
      </Flex>
    </Flex>
  );
}
