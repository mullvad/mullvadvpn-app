import styled from 'styled-components';

import { spacings } from '../../../../foundations';
import { Flex, FlexProps } from '../../../flex';

export type ViewContentProps = FlexProps;

export const StyledViewContent = styled(Flex)`
  height: 100%;
  margin-bottom: ${spacings.large};
`;

export function ViewContent(props: ViewContentProps) {
  return <StyledViewContent flexDirection="column" flexGrow={1} {...props} />;
}
