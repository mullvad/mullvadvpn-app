import React from 'react';

import { AccordionProvider } from './AccordionContext';
import {
  AccordionContainer,
  AccordionContent,
  AccordionHeader,
  AccordionHeaderItem,
  AccordionIcon,
  AccordionTitle,
  AccordionTrigger,
} from './components';

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
  Trigger: AccordionTrigger,
  Header: AccordionHeader,
  HeaderItem: AccordionHeaderItem,
  Content: AccordionContent,
  Title: AccordionTitle,
  Icon: AccordionIcon,
});

export { AccordionNamespace as Accordion };
