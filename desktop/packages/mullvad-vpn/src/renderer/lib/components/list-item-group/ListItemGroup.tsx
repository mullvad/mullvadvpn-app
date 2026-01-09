import styled, { css } from 'styled-components';

import { Flex, FlexProps } from '../flex';
import { StyledListItemItem } from '../list-item/components';
import { StyledListItem } from '../list-item/ListItem';

type ListItemGroupVariant = 'default' | 'grouped';
type UnsetCornerRadius = 'top' | 'bottom' | 'both';

export type ListItemGroupProps = FlexProps & {
  variant?: ListItemGroupVariant;
  unsetCornerRadius?: UnsetCornerRadius;
};

const StyledListItemGroup = styled(Flex)<{
  $variant?: ListItemGroupVariant;
  $unsetCornerRadius?: UnsetCornerRadius;
  $customGap?: boolean;
}>`
  ${({ $variant, $unsetCornerRadius, $customGap }) => {
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
            // If it contains a list item that is not the first child, remove top border radius
            && > ${StyledListItem}:nth-child(n + 2) ${StyledListItemItem} {
              border-top-left-radius: 0;
              border-top-right-radius: 0;
            }

            // If it contains a list item that is followed by another list item, remove bottom border radius
            && > ${StyledListItem}:has(+ ${StyledListItem}) ${StyledListItemItem} {
              border-bottom-left-radius: 0;
              border-bottom-right-radius: 0;
            }
          `;
        }
        return null;
      }}

      ${() => {
        if ($unsetCornerRadius === 'bottom' || $unsetCornerRadius === 'both') {
          return css`
            // Remove bottom border radius from the last list item
            && > ${StyledListItem}:last-of-type ${StyledListItemItem} {
              border-bottom-left-radius: 0;
              border-bottom-right-radius: 0;
            }
          `;
        }
        return null;
      }}
      
      ${() => {
        if ($unsetCornerRadius === 'top' || $unsetCornerRadius === 'both') {
          return css`
            // Remove top border radius from the first list item
            && > ${StyledListItem}:first-of-type ${StyledListItemItem} {
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

export function ListItemGroup({
  variant = 'default',
  unsetCornerRadius,
  gap,
  ...props
}: ListItemGroupProps) {
  const customGap = gap !== undefined;
  return (
    <StyledListItemGroup
      flexDirection="column"
      gap={gap}
      $variant={variant}
      $unsetCornerRadius={unsetCornerRadius}
      $customGap={customGap}
      {...props}
    />
  );
}
