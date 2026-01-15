import React from 'react';

import { AccordionProvider } from './AccordionContext';
import {
  AccordionContent,
  AccordionHeader,
  AccordionHeaderItem,
  AccordionIcon,
  AccordionTitle,
  AccordionTrigger,
} from './components';

export type AccordionAnimation = 'flash' | 'dim';

export type AccordionProps = React.PropsWithChildren<{
  expanded?: boolean;
  onExpandedChange?: (open: boolean) => void;
  disabled?: boolean;
  titleId?: string;
}>;

function Accordion({
  expanded = false,
  onExpandedChange: onOpenChange,
  disabled,
  titleId: titleIdProp,
  children,
}: AccordionProps) {
  const triggerId = React.useId();
  const contentId = React.useId();
  const titleId = React.useId();
  return (
    <AccordionProvider
      triggerId={triggerId}
      contentId={contentId}
      titleId={titleIdProp ?? titleId}
      expanded={expanded}
      onExpandedChange={onOpenChange}
      disabled={disabled}>
      <div>{children}</div>
    </AccordionProvider>
  );
}

const AccordionNamespace = Object.assign(Accordion, {
  Trigger: AccordionTrigger,
  Header: AccordionHeader,
  HeaderItem: AccordionHeaderItem,
  Content: AccordionContent,
  Title: AccordionTitle,
  Icon: AccordionIcon,
});

export { AccordionNamespace as Accordion };
