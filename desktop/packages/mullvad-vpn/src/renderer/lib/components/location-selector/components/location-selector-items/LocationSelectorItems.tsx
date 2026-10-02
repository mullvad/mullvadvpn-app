import { AnimatePresence, type AnimatePresenceProps, LayoutGroup, motion } from 'motion/react';
import styled from 'styled-components';

import { colors, Radius, spacings } from '../../../../foundations';
import type { LocationSelectorVariant } from '../../LocationSelector';
import { useLocationSelectorContext } from '../../LocationSelectorContext';
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
  border-radius: ${Radius.radius16};
  padding: ${spacings.tiny};
  gap: ${spacings.tiny};
`;

function LocationSelectorItems({ children }: LocationSelectorItemsProps) {
  const { expanded, variant } = useLocationSelectorContext();

  return (
    <LayoutGroup>
      <StyledLocationSelectorItems
        layout="preserve-aspect"
        $variant={variant}
        initial={false}
        animate={{
          backgroundColor: variant === 'primary' ? colors.darkBlue : colors.darkerBlue10,
        }}
        transition={{
          layout: { duration: 5, ease: 'easeOut' },
          backgroundColor: { duration: 0.15, ease: 'linear' },
        }}>
        <LocationSelectorLine
          $visible={expanded}
          layout
          transition={{ duration: 0.25, ease: 'easeOut' }}
        />
        <AnimatePresence mode="popLayout">{children}</AnimatePresence>
      </StyledLocationSelectorItems>
    </LayoutGroup>
  );
}

const LocationSelectorItemsNamespace = Object.assign(LocationSelectorItems, {
  TextFieldItem: LocationSelectorTextFieldItem,
  ButtonItem: LocationSelectorButtonItem,
});

export { LocationSelectorItemsNamespace as LocationSelectorItems };
