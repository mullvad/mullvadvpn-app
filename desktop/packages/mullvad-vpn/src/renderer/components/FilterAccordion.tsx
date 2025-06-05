import React from 'react';

import { Accordion } from '../lib/components/accordion';

export type FilterAccordionProps = {
  title?: string;
  children?: React.ReactNode;
  defaultOpen?: boolean;
};

export function FilterAccordion({ title, children, defaultOpen }: FilterAccordionProps) {
  const [open, setOpen] = React.useState(defaultOpen);
  return (
    <Accordion expanded={open} onExpandedChange={setOpen}>
      <Accordion.Trigger>
        <Accordion.Header>
          <Accordion.Title>{title}</Accordion.Title>
          <Accordion.Icon />
        </Accordion.Header>
      </Accordion.Trigger>
      <Accordion.Content>{children}</Accordion.Content>
    </Accordion>
  );
}
