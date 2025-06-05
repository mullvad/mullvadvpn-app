import React from 'react';
import styled from 'styled-components';

import { AccordionProvider } from './AccordionContext';
import { AccordionHeader, AccordionTrigger } from './components';
import { AccordionContent } from './components/AccordionContent';
import { AccordionIcon } from './components/AccordionIcon';
import { AccordionTitle } from './components/AccordionTitle';

export type AccordionProps = {
  expanded?: boolean;
  onExpandedChange?: (open: boolean) => void;
  children?: React.ReactNode;
};

const StyledAccordion = styled.div`
  display: flex;
  flex: 1;
  flex-direction: column;
  width: 100%;
`;

function Accordion({ expanded = false, onExpandedChange: onOpenChange, children }: AccordionProps) {
  const triggerId = React.useId();
  const contentId = React.useId();
  return (
    <AccordionProvider
      triggerId={triggerId}
      contentId={contentId}
      expanded={expanded}
      onExpandedChange={onOpenChange}>
      <StyledAccordion>{children}</StyledAccordion>
    </AccordionProvider>
  );
}

const AccordionNamespace = Object.assign(Accordion, {
  Trigger: AccordionTrigger,
  Header: AccordionHeader,
  Content: AccordionContent,
  Title: AccordionTitle,
  Icon: AccordionIcon,
});

export { AccordionNamespace as Accordion };
