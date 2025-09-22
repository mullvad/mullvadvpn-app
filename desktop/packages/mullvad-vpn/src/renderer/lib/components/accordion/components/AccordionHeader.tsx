import styled, { css, RuleSet } from 'styled-components';

import { colors } from '../../../foundations';
import { Flex } from '../../flex';
import { ListItem } from '../../list-item';

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
  return (
    <ListItem.Item>
      <ListItem.Content>{children}</ListItem.Content>
    </ListItem.Item>
  );
}
