import { AnimatePresence, motion } from 'motion/react';
import React from 'react';
import styled from 'styled-components';

import { useAccordionContext } from '../AccordionContext';

export type AccordionContentProps = {
  children?: React.ReactNode;
};

const StyledAccordionContent = styled(motion.div)`
  width: 100%;
  overflow: hidden;
`;

const variants = {
  collapsed: { height: 0, opacity: 0 },
  expanded: { height: 'auto', opacity: 1 },
};

export function AccordionContent({ children }: AccordionContentProps) {
  const { contentId, triggerId, expanded } = useAccordionContext();
  return (
    <AnimatePresence initial={false}>
      {expanded && (
        <StyledAccordionContent
          id={contentId}
          aria-labelledby={triggerId}
          layout
          role="region"
          variants={variants}
          initial="collapsed"
          animate="expanded"
          exit="collapsed"
          transition={{ duration: 0.25, ease: 'easeInOut' }}>
          {children}
        </StyledAccordionContent>
      )}
    </AnimatePresence>
  );
}
