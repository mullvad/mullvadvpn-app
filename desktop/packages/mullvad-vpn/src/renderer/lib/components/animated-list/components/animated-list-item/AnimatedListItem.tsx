import { HTMLMotionProps, motion } from 'motion/react';
import styled from 'styled-components';

export type AnimatedListItemProps = HTMLMotionProps<'li'>;

const StyledLi = styled(motion.li)`
  overflow: hidden;
`;

const itemVariants = {
  hidden: { height: 0, opacity: 0 },
  show: { height: 'auto', opacity: 1 },
  exit: { height: 0, opacity: 0 },
};

export function AnimatedListItem({ children, ...props }: AnimatedListItemProps) {
  return (
    <StyledLi
      variants={itemVariants}
      initial="hidden"
      animate="show"
      exit="exit"
      transition={{ duration: 0.15, ease: 'easeOut' }}
      {...props}>
      {children}
    </StyledLi>
  );
}
