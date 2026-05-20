import styled, { css } from 'styled-components';

import { colors, Radius, spacings } from '../../../../foundations';
import { AnimatedList } from '../../../animated-list';
import type { LocationSelectorVariant } from '../../LocationSelector';
import { useLocationSelectorContext } from '../../LocationSelectorContext';
import { LocationSelectorLine } from '../location-selector-line';
import { LocationSelectorItem, StyledLocationSelectorItem } from './components';

export type LocationSelectorItemsProps = React.PropsWithChildren;

export const StyledLocationSelectorItems = styled(AnimatedList)<{
  $variant?: LocationSelectorVariant;
}>`
  ${({ $variant }) => {
    return css`
      position: relative;
      display: flex;
      flex-direction: column;
      background-color: ${$variant === 'primary' ? colors.transparent : colors.darkerBlue10};
      padding: ${spacings.small} ${spacings.small} ${spacings.tiny} ${spacings.small};
      border-radius: ${Radius.radius16};

      transition: background-color 0.15s ease-in-out;
      transition-delay: 0s;
      &:has(> ${StyledLocationSelectorItem}:nth-last-of-type(n+2)) {
        transition-delay: 0.15s;
      }
    `;
  }}
`;

function LocationSelectorItems({ children }: LocationSelectorItemsProps) {
  const { expanded, variant } = useLocationSelectorContext();

  return (
    <StyledLocationSelectorItems $variant={variant}>
      <LocationSelectorLine $visible={expanded} />
      {children}
    </StyledLocationSelectorItems>
  );
}

const LocationSelectorItemsNamespace = Object.assign(LocationSelectorItems, {
  Item: LocationSelectorItem,
});

export { LocationSelectorItemsNamespace as LocationSelectorItems };
