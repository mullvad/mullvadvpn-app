import { motion } from 'motion/react';
import styled from 'styled-components';

export type AnimatedListItemProps = React.ComponentPropsWithRef<'li'>;

const StyledLi = styled(motion.li)`
  overflow: hidden;
`;

const itemVariants = {
  hidden: { height: 0 },
  show: { height: 'auto' },
  exit: { height: 0 },
};

export function AnimatedListItem({ children }: AnimatedListItemProps) {
  return (
    <StyledLi
      layout
      variants={itemVariants}
      initial="hidden"
      animate="show"
      exit="exit"
      transition={{ duration: 0.15, ease: 'easeOut' }}>
      {children}
    </StyledLi>
  );
}
