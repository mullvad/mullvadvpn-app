import { type HTMLMotionProps, motion } from 'motion/react';
import styled from 'styled-components';

export type ExpandableContentProps = HTMLMotionProps<'div'>;

export const StyledExpandableContent = styled(motion.div)`
  width: 100%;
  overflow: hidden;
`;

const variants = {
  collapsed: { height: 0, opacity: 0 },
  expanded: { height: 'auto', opacity: 1 },
};

export function ExpandableContent({ children, ...props }: ExpandableContentProps) {
  return (
    <StyledExpandableContent
      initial="collapsed"
      animate="expanded"
      exit="collapsed"
      variants={variants}
      transition={{ duration: 0.25, ease: 'easeInOut' }}
      {...props}>
      {children}
    </StyledExpandableContent>
  );
}
