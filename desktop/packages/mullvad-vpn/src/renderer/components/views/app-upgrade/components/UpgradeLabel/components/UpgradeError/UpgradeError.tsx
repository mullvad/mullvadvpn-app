import { Flex, LabelTiny } from '../../../../../../../lib/components';
import { Dot } from '../../../../../../../lib/components/dot';
import { useTexts } from './hooks';

export function UpgradeError() {
  const texts = useTexts();

  return (
    <Flex $gap="small" $alignItems="baseline" $flexDirection="row">
      <Dot variant="error" />
      <Flex $flexDirection="column">
        {texts.map((text) => (
          <LabelTiny key={text}>{text}</LabelTiny>
        ))}
      </Flex>
    </Flex>
  );
}
