import { AnimatePresence, type AnimatePresenceProps, motion } from 'motion/react';
import styled, { css } from 'styled-components';

import { colors, Radius, spacings } from '../../../../foundations';
import type { LocationSelectorVariant } from '../../LocationSelector';
import { useLocationSelectorContext } from '../../LocationSelectorContext';
import { LocationSelectorLine } from '../location-selector-line';
import { LocationSelectorButtonItem, LocationSelectorTextFieldItem } from './components';

export type LocationSelectorItemsProps = AnimatePresenceProps & React.PropsWithChildren;

export const StyledLocationSelectorItems = styled(motion.div)<{
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
      <AnimatePresence mode="popLayout">
        <LocationSelectorLine $visible={expanded} />
        {children}
      </AnimatePresence>
    </StyledLocationSelectorItems>
  );
}

const LocationSelectorItemsNamespace = Object.assign(LocationSelectorItems, {
  TextFieldItem: LocationSelectorTextFieldItem,
  ButtonItem: LocationSelectorButtonItem,
});

export { LocationSelectorItemsNamespace as LocationSelectorItems };
