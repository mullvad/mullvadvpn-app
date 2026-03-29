import React from 'react';
import styled from 'styled-components';

import { Expandable } from '../../expandable';
import { useAccordionContext } from '../AccordionContext';

export type AccordionContentProps = {
  children?: React.ReactNode;
};

export const StyledAccordionContent = styled(Expandable.Content)``;

export function AccordionContent({ children, ...props }: AccordionContentProps) {
  const { contentId, triggerId, expanded, setContent } = useAccordionContext();

  return (
    <Expandable expanded={expanded} {...props}>
      <StyledAccordionContent id={contentId} ref={setContent} aria-labelledby={triggerId}>
        {children}
      </StyledAccordionContent>
    </Expandable>
  );
}
