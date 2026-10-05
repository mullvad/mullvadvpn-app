import { type HTMLMotionProps, motion } from 'motion/react';
import styled from 'styled-components';

import { spacings } from '../../../../../../foundations';

export type LocationSelectorItemProps = HTMLMotionProps<'div'>;

const StyledLocationSelectorItem = styled(motion.div)`
  z-index: var(--location-selector-item-z-index);
  overflow: hidden;
  display: flex;
  width: 100%;
  overflow: hidden;

  margin-bottom: ${spacings.tiny};
`;

export function LocationSelectorItem({ children, ...props }: LocationSelectorItemProps) {
  return (
    <StyledLocationSelectorItem
      layout="preserve-aspect"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.15, ease: 'easeOut' }}
      {...props}>
      {children}
    </StyledLocationSelectorItem>
  );
}
