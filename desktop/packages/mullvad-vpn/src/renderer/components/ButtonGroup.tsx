import styled from 'styled-components';

import { Flex, FlexProps } from '../lib/components';
import { StyledButton } from '../lib/components/button';
import { StyledButtonText } from '../lib/components/button/components';

export type WrapButtonGroupProps = FlexProps;

const StyledFlex = styled(Flex)`
  && > ${StyledButton} {
    flex: 1 0 0;
    min-width: auto;
    max-width: 100%;
    white-space: nowrap;
    & > ${StyledButtonText} {
      overflow: hidden;
      text-overflow: ellipsis;
    }
  }
`;

export function ButtonGroup({ $gap, children, ...props }: WrapButtonGroupProps) {
  return (
    <StyledFlex $gap={$gap} $flexWrap="wrap" {...props}>
      {children}
    </StyledFlex>
  );
}
