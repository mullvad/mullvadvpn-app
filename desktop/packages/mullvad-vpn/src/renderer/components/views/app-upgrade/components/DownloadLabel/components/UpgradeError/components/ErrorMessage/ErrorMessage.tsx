import styled from 'styled-components';

import { Flex, LabelTiny } from '../../../../../../../../../lib/components';
import { Colors } from '../../../../../../../../../lib/foundations';
import { useTexts } from './hooks';

// TODO: If we add different variants we should actually use them
// maybe add this as a generic component?
const Indicator = styled.div<{ $variant?: 'error' | 'warning' }>`
  min-width: 10px;
  min-height: 10px;
  border-radius: 50%;
  background-color: ${({ $variant }) => ($variant === 'error' ? Colors.red : Colors.yellow)};
`;

export function ErrorMessage() {
  const texts = useTexts();

  return (
    <Flex $gap="small" $alignItems="baseline" $flexDirection="row">
      <Indicator $variant="error" />
      <Flex $flexDirection="column">
        {texts.map((text) => (
          <LabelTiny key={text}>{text}</LabelTiny>
        ))}
      </Flex>
    </Flex>
  );
}
