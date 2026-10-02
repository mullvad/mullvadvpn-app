import { type HTMLMotionProps, motion } from 'motion/react';
import styled from 'styled-components';

export type LocationSelectorItemProps = HTMLMotionProps<'div'>;

export const StyledLocationSelectorItem = styled(motion.div)`
  z-index: var(--line-z-index);
  overflow: hidden;
  display: flex;
  width: 100%;
`;

export function LocationSelectorItem({ children, ...props }: LocationSelectorItemProps) {
  return (
    <StyledLocationSelectorItem
      layout="preserve-aspect"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.25, ease: 'easeOut' }}
      {...props}>
      {children}
    </StyledLocationSelectorItem>
  );
}
