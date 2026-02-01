import styled, { css } from 'styled-components';

import { Flex, Text, type TextProps } from '../../../../lib/components';
import { ListItem } from '../../../../lib/components/list-item';
import { useLocationListItemContext } from '../../LocationListItemContext';
import { useListItemLabelColor } from './hooks';

export type LocationAccordionTitleProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const StyledFlex = styled(Flex)`
  position: relative;
`;

export const StyledLocationAccordionTitleIcon = styled(ListItem.Icon)<{ $selected: boolean }>`
  ${({ $selected }) => {
    return css`
      --transition-duration: 0.15s;

      position: absolute;
      transition:
        opacity var(--transition-duration) ease-in,
        transform var(--transition-duration) ease-out;
      transform: ${$selected ? 'translateX(0)' : 'translateX(-16px)'};
      opacity: ${$selected ? 1 : 0};
      visibility: ${$selected ? 'visible' : 'hidden'};
    `;
  }}
`;

const StyledLocationAccordionTitle = styled(Text)<{ $selected: boolean }>`
  ${({ $selected }) => css`
    --transition-duration: 0.15s;

    transition:
      transform var(--transition-duration) ease-out,
      color var(--transition-duration) ease-out;
    transform: translateX(${$selected ? 32 : 0}px);
  `}
`;

export const LocationAccordionTitle = <E extends React.ElementType = 'span'>(
  props: LocationAccordionTitleProps<E>,
) => {
  const color = useListItemLabelColor();
  const { selected = false } = useLocationListItemContext();
  return (
    <StyledFlex>
      <StyledLocationAccordionTitleIcon icon="checkmark" color={color} $selected={selected} />
      <StyledLocationAccordionTitle
        key="label"
        variant="bodySmallSemibold"
        color={color}
        $selected={selected}
        {...props}
      />
    </StyledFlex>
  );
};
