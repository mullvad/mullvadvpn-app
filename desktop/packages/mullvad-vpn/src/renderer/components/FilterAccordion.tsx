import React from 'react';
import styled from 'styled-components';

import { Accordion, AccordionProps } from '../lib/components/accordion';

export type FilterAccordionProps = AccordionProps & {
  title?: string;
  defaultOpen?: boolean;
};

const StyledAccordionHeader = styled(Accordion.Header)`
  margin-bottom: 1px;
`;

export function FilterAccordion({ title, children, defaultOpen }: FilterAccordionProps) {
  const [open, setOpen] = React.useState(defaultOpen);
  return (
    <Accordion expanded={open} onExpandedChange={setOpen}>
      <Accordion.Container>
        <StyledAccordionHeader>
          <Accordion.Trigger>
            <Accordion.HeaderItem>
              <Accordion.Title>{title}</Accordion.Title>
              <Accordion.HeaderActionGroup>
                <Accordion.Icon />
              </Accordion.HeaderActionGroup>
            </Accordion.HeaderItem>
          </Accordion.Trigger>
        </StyledAccordionHeader>
        <Accordion.Content>{children}</Accordion.Content>
      </Accordion.Container>
    </Accordion>
  );
}
