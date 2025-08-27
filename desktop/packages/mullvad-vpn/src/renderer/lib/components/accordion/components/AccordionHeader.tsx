import styled, { css, RuleSet } from 'styled-components';

import { colors } from '../../../foundations';
import { Flex } from '../../flex';
import { useAccordionContext } from '../AccordionContext';
import { useAnimation } from '../hooks';

export type AccordionHeaderProps = {
  children?: React.ReactNode;
};

export const StyledAccordionHeader = styled(Flex)<{
  $animation?: RuleSet<object>;
  $disabled?: boolean;
}>`
  ${({ $animation, $disabled }) => {
    const backgroundColor = $disabled ? colors.blue40 : colors.blue;
    return css`
      --background-color: ${backgroundColor};

      margin-bottom: 1px;
      background-color: var(--background-color);
      min-height: 48px;
      width: 100%;
      ${$animation}
    `;
  }}
`;

export function AccordionHeader({ children }: AccordionHeaderProps) {
  const animation = useAnimation();
  const { disabled } = useAccordionContext();
  return (
    <StyledAccordionHeader
      $padding={{ horizontal: 'medium', vertical: 'small' }}
      $alignItems="center"
      $justifyContent="space-between"
      $animation={animation}
      $disabled={disabled}>
      {children}
    </StyledAccordionHeader>
  );
}
