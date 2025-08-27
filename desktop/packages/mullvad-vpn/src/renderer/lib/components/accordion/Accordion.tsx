import React from 'react';
import styled from 'styled-components';

import { FlexColumn } from '../flex-column';
import { AccordionProvider } from './AccordionContext';
import { AccordionHeader, AccordionTrigger } from './components';
import { AccordionContent } from './components/AccordionContent';
import { AccordionIcon } from './components/AccordionIcon';
import { AccordionTitle } from './components/AccordionTitle';

export type AccordionAnimation = 'flash' | 'dim';

export type AccordionProps = {
  expanded?: boolean;
  onExpandedChange?: (open: boolean) => void;
  disabled?: boolean;
  animation?: AccordionAnimation;
  children?: React.ReactNode;
} & React.ComponentPropsWithRef<'div'>;

const StyledAccordion = styled(FlexColumn)`
  width: 100%;
`;

function Accordion({
  expanded = false,
  onExpandedChange: onOpenChange,
  disabled,
  animation,
  children,
  ...props
}: AccordionProps) {
  const triggerId = React.useId();
  const contentId = React.useId();
  const titleId = React.useId();
  return (
    <AccordionProvider
      triggerId={triggerId}
      contentId={contentId}
      titleId={titleId}
      expanded={expanded}
      onExpandedChange={onOpenChange}
      disabled={disabled}
      animation={animation}>
      <StyledAccordion $flex={1} {...props}>
        {children}
      </StyledAccordion>
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
