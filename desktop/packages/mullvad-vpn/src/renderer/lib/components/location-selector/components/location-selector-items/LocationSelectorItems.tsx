import styled, { css } from 'styled-components';

import { colors, Radius, spacings } from '../../../../foundations';
import { AnimatedList } from '../../../animated-list';
import type { LocationSelectorVariant } from '../../LocationSelector';
import { useLocationSelectorContext } from '../../LocationSelectorContext';
import { LocationSelectorLine } from '../location-selector-line';
import { LocationSelectorItem } from './components';

export type LocationSelectorItemsProps = React.PropsWithChildren;

export const StyledLocationSelectorItems = styled(AnimatedList)<{
  $variant?: LocationSelectorVariant;
}>`
  ${({ $variant }) => {
    return css`
      position: relative;
      display: flex;
      align-items: center;
      flex-direction: column;
      background-color: ${$variant === 'primary' ? colors.darkBlue : colors.darkerBlue10};
      padding-top: ${spacings.tiny};
      border-radius: ${Radius.radius16};
      transition: background-color 0.15s ease-in-out;
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
