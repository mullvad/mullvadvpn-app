import styled, { css } from 'styled-components';

import { colors, Radius, spacings } from '../../../../foundations';
import { AnimatedList } from '../../../animated-list';
import type { LocationSelectorVariant } from '../../LocationSelector';
import { useLocationSelectorContext } from '../../LocationSelectorContext';
import { LocationSelectorLine } from '../location-selector-line';
import { LocationSelectorItem } from './components';
import { StyledLocationSelectorTrailingButton } from './components/location-selector-item/components';

export type LocationSelectorItemsProps = React.PropsWithChildren;

export const StyledLocationSelectorItems = styled(AnimatedList)<{
  $variant?: LocationSelectorVariant;
}>`
  ${({ $variant }) => {
    return css`
      position: relative;
      display: flex;
      flex-direction: column;
      background-color: ${$variant === 'primary' ? colors.darkerBlue10 : colors.darkerBlue10};
      padding: ${spacings.tiny} ${spacings.tiny} 0 ${spacings.tiny};
      border-radius: ${Radius.radius16};
      transition: background-color 0.15s ease-in-out;

      // Padding is built in to the trailing button, so remove padding
      // when it is present to ensure the spacing is correct
      &:has(${StyledLocationSelectorTrailingButton}) {
        padding-right: 0px;
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
