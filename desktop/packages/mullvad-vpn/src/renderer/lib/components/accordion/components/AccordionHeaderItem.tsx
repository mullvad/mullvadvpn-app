import styled, { css } from 'styled-components';

import { ListItem } from '../../list-item';
import { ListItemItemProps } from '../../list-item/components';
import { useAccordionContext } from '../AccordionContext';

export type AccordionHeaderItemProps = ListItemItemProps;

export const StyledAccordionHeaderItem = styled(ListItem.Item)<{ $expanded?: boolean }>`
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

export function AccordionHeaderItem({ children, ...props }: AccordionHeaderItemProps) {
  const { expanded } = useAccordionContext();
  return (
    <StyledAccordionHeaderItem $expanded={expanded} {...props}>
      <ListItem.Content>{children}</ListItem.Content>
    </StyledAccordionHeaderItem>
  );
}
