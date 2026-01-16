import React from 'react';

import { Accordion, AccordionProps } from '../lib/components/accordion';

export type FilterAccordionProps = AccordionProps & {
  title?: string;
  defaultOpen?: boolean;
};

export function FilterAccordion({ title, children, defaultOpen }: FilterAccordionProps) {
  const [open, setOpen] = React.useState(defaultOpen);
  return (
    <Accordion expanded={open} onExpandedChange={setOpen}>
      <Accordion.Header>
        <Accordion.Trigger>
          <Accordion.HeaderItem>
            <Accordion.Title>{title}</Accordion.Title>
            <Accordion.Icon />
          </Accordion.HeaderItem>
        </Accordion.Trigger>
      </Accordion.Header>
      <Accordion.Content>{children}</Accordion.Content>
    </Accordion>
  );
}
