import { AnimatePresence, type AnimatePresenceProps, motion } from 'motion/react';
import styled from 'styled-components';

import { colors, Radius, spacings } from '../../../../foundations';
import { useLocationSelectorContext } from '../../LocationSelectorContext';
import type { LocationSelectorVariant } from '../../types';
import { LocationSelectorLine } from '../location-selector-line';
import { LocationSelectorButtonItem, LocationSelectorTextFieldItem } from './components';

export type LocationSelectorItemsProps = AnimatePresenceProps & React.PropsWithChildren;

export const StyledLocationSelectorItems = styled(motion.div)<{
  $variant?: LocationSelectorVariant;
}>`
  position: relative;
  display: flex;
  align-items: center;
  flex-direction: column;
  padding-top: ${spacings.tiny};
  border-radius: ${Radius.radius16};
`;

function LocationSelectorItems({ children }: LocationSelectorItemsProps) {
  const { expanded, variant } = useLocationSelectorContext();

  return (
    <StyledLocationSelectorItems
      layout="preserve-aspect"
      $variant={variant}
      initial={false}
      animate={{
        backgroundColor: variant === 'primary' ? colors.darkBlue : colors.darkerBlue10,
      }}
      transition={{ duration: 0.15, ease: 'linear' }}>
      <LocationSelectorLine
        $visible={expanded}
        layout
        transition={{ duration: 0.15, ease: 'easeOut' }}
      />
      <AnimatePresence mode="popLayout">{children}</AnimatePresence>
    </StyledLocationSelectorItems>
  );
}

const LocationSelectorItemsNamespace = Object.assign(LocationSelectorItems, {
  TextFieldItem: LocationSelectorTextFieldItem,
  ButtonItem: LocationSelectorButtonItem,
});

export { LocationSelectorItemsNamespace as LocationSelectorItems };
