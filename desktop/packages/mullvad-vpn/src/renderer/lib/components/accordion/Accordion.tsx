import React from 'react';

import { AccordionProvider } from './AccordionContext';
import { AccordionContainer, AccordionContent, AccordionHeader } from './components';

export type AccordionProps = React.PropsWithChildren<{
  expanded?: boolean;
  onExpandedChange?: (open: boolean) => void;
  disabled?: boolean;
  titleId?: string;
}>;

function Accordion({
  expanded = false,
  onExpandedChange,
  disabled,
  titleId,
  children,
}: AccordionProps) {
  return (
    <AccordionProvider
      titleId={titleId}
      expanded={expanded}
      onExpandedChange={onExpandedChange}
      disabled={disabled}>
      {children}
    </AccordionProvider>
  );
}

const AccordionNamespace = Object.assign(Accordion, {
  Container: AccordionContainer,
  Header: AccordionHeader,
  Content: AccordionContent,
});

export { AccordionNamespace as Accordion };
