import { AnimatePresence, type AnimatePresenceProps, motion } from 'motion/react';
import styled from 'styled-components';

import { colors, Radius, spacings } from '../../../../foundations';
import type { LocationSelectorVariant } from '../../LocationSelector';
import { useLocationSelectorContext } from '../../LocationSelectorContext';
import { LocationSelectorLine } from '../location-selector-line';
import {
  LocationSelectorButtonItem,
  LocationSelectorTextFieldItem,
  LocationSelectorTrailingButton,
} from './components';

export type LocationSelectorItemsProps = AnimatePresenceProps & React.PropsWithChildren;

export const StyledLocationSelectorItems = styled(motion.div)<{
  $variant?: LocationSelectorVariant;
}>`
  position: relative;
  display: flex;
  align-items: center;
  flex-direction: column;
  border-radius: ${Radius.radius16};
  padding: ${spacings.tiny};
  gap: ${spacings.tiny};
`;

function LocationSelectorItems({ children }: LocationSelectorItemsProps) {
  const { expanded, variant } = useLocationSelectorContext();

  return (
    <StyledLocationSelectorItems
      layout="size"
      $variant={variant}
      initial={false}
      animate={{
        backgroundColor: variant === 'primary' ? colors.darkBlue : colors.darkerBlue10,
      }}
      transition={{
        layout: { duration: 0.25, ease: 'easeOut' },
        backgroundColor: { duration: 0.15, ease: 'linear' },
      }}>
      <LocationSelectorLine
        $visible={expanded}
        layout
        transition={{ duration: 0.25, ease: 'easeOut' }}
      />
      <AnimatePresence mode="popLayout">{children}</AnimatePresence>
    </StyledLocationSelectorItems>
  );
}

const LocationSelectorItemsNamespace = Object.assign(LocationSelectorItems, {
  TextFieldItem: LocationSelectorTextFieldItem,
  TrailingButton: LocationSelectorTrailingButton,
  ButtonItem: LocationSelectorButtonItem,
});

export { LocationSelectorItemsNamespace as LocationSelectorItems };
