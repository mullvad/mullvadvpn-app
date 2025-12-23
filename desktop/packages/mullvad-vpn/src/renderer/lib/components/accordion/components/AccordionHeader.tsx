import styled, { css } from 'styled-components';

import { ListItem } from '../../list-item';
import { ListItemItemProps } from '../../list-item/components';
import { useAccordionContext } from '../AccordionContext';

export type AccordionHeaderProps = ListItemItemProps;

export const StyledAccordionHeader = styled(ListItem.Item)<{ $expanded?: boolean }>`
  ${({ $expanded }) => {
    return css`
      transition: border-radius 0.15s ease-out;
      ${() => {
        if ($expanded) {
          return css`
            border-bottom-right-radius: 0;
            border-bottom-left-radius: 0;
          `;
        }
        return null;
      }}
    `;
  }}
`;

export function AccordionHeader({ children, ...props }: AccordionHeaderProps) {
  const { expanded } = useAccordionContext();
  return (
    <StyledAccordionHeader $expanded={expanded} {...props}>
      <ListItem.Content>{children}</ListItem.Content>
    </StyledAccordionHeader>
  );
}
