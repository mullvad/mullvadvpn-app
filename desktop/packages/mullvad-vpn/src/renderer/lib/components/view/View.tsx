import styled from 'styled-components';

import { Colors, colors, ColorVariables } from '../../foundations';
import { Flex, FlexProps } from '../flex';
import { Container } from './components';

export type ViewProps = FlexProps & {
  backgroundColor?: Colors;
};

export const StyledView = styled(Flex)<{ $backgroundColor?: ColorVariables }>`
  height: 100vh;
  max-width: 100%;
  background-color: ${({ $backgroundColor }) => $backgroundColor || undefined};
`;

function View({ backgroundColor = 'blue', ...props }: ViewProps) {
  return (
    <StyledView $backgroundColor={colors[backgroundColor]} flexDirection="column" {...props} />
  );
}

const ViewNamespace = Object.assign(View, {
  Container: Container,
});

export { ViewNamespace as View };
