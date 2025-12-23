import styled, { css } from 'styled-components';

import { Flex, FlexProps } from '../flex';
import { StyledListItemItem } from '../list-item/components';
import { StyledListItem } from '../list-item/ListItem';

type ListItemGroupVariant = 'default' | 'grouped';

export type ListItemGroupProps = FlexProps & {
  variant?: ListItemGroupVariant;
};

const StyledListItemGroup = styled(Flex)<{ $variant?: ListItemGroupVariant; $customGap?: boolean }>`
  ${({ $customGap, $variant }) => {
    return css`
      ${() => {
        if (!$customGap) {
          return css`
            gap: 1px;
          `;
        }
        return null;
      }}
      ${() => {
        if ($variant === 'grouped') {
          return css`
            // If it contains a list item that is followed by another list item, remove bottom border radius
            && > ${StyledListItem}:has(+ ${StyledListItem}) ${StyledListItemItem} {
              border-bottom-left-radius: 0;
              border-bottom-right-radius: 0;
            }

            // If it contains a list item that is not the first child, remove top border radius
            && > ${StyledListItem}:nth-child(n + 2) ${StyledListItemItem} {
              border-top-left-radius: 0;
              border-top-right-radius: 0;
            }
          `;
        }
        return null;
      }}
    `;
  }}
`;

export function ListItemGroup({ variant = 'default', gap, ...props }: ListItemGroupProps) {
  const customGap = gap !== undefined;
  return (
    <StyledListItemGroup
      flexDirection="column"
      gap={gap}
      $variant={variant}
      $customGap={customGap}
      {...props}
    />
  );
}
