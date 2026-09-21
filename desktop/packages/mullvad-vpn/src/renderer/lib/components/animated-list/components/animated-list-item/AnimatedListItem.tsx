import { HTMLMotionProps, motion } from 'motion/react';
import styled from 'styled-components';

export type AnimatedListItemProps = HTMLMotionProps<'li'>;

const StyledLi = styled(motion.li)`
  overflow: hidden;
`;

export function AnimatedListItem({ children, ...props }: AnimatedListItemProps) {
  return (
    <StyledLi
      initial={{ height: 0, opacity: 0 }}
      animate={{ height: 'auto', opacity: 1 }}
      exit={{ height: 0, opacity: 0 }}
      transition={{ duration: 0.25, ease: 'easeOut' }}
      {...props}>
      {children}
    </StyledLi>
  );
}
