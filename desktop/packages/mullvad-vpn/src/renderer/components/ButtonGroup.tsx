import styled, { css } from 'styled-components';

import { Flex, FlexProps } from '../lib/components';
import { StyledButton } from '../lib/components/button';

export type WrapButtonGroupProps = FlexProps;

const StyledFlex = styled(Flex)`
  ${({ gap: gapProp }) => {
    const $gap = gapProp ?? '0px';
    return css`
      && > ${StyledButton} {
        flex: 1 0 auto;
        max-width: 100%;
      }
      &&:has(> :nth-child(2)) {
        & > ${StyledButton} {
          min-width: calc((100% - ${$gap}) / 2);
        }
      }
      &&:has(> :nth-child(3)) {
        & > ${StyledButton} {
          min-width: calc((100% - ${$gap} * 2) / 3);
        }
      }
      &&:has(> :nth-child(4)) {
        & > ${StyledButton} {
          min-width: calc((100% - ${$gap} * 3) / 4);
        }
      }
      &&:has(> :nth-child(5)) {
        & > ${StyledButton} {
          min-width: calc((100% - ${$gap} * 4) / 5);
        }
      }
    `;
  }}
`;

export function ButtonGroup({ gap, children, ...props }: WrapButtonGroupProps) {
  return (
    <StyledFlex gap={gap} flexWrap="wrap" {...props}>
      {children}
    </StyledFlex>
  );
}
