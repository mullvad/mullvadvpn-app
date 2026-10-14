import { HTMLMotionProps, motion } from 'motion/react';
import styled from 'styled-components';

import { durationTokens } from '../../../../foundations';

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
      transition={{ duration: durationTokens.normal, ease: 'easeOut' }}
      {...props}>
      {children}
    </StyledLi>
  );
}
