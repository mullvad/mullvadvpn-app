import styled from 'styled-components';

import { Flex } from '../../../../../lib/components';
import { colors, Radius } from '../../../../../lib/foundations';
import { useContent } from './components/hooks';

const StyledLaunchFooter = styled(Flex)`
  width: 100%;
  background-color: ${colors.darkBlue};
  border-radius: ${Radius.radius8};
`;

export function Footer() {
  const content = useContent();
  return (
    <StyledLaunchFooter flexDirection="column" padding="medium">
      {content}
    </StyledLaunchFooter>
  );
}
