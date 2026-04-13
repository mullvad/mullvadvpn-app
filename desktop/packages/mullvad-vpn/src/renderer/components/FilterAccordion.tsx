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
          <Accordion.Header.AccordionTrigger>
            <Accordion.Header.Item>
              <Accordion.Header.Item.Title>{title}</Accordion.Header.Item.Title>
              <Accordion.Header.Item.ActionGroup>
                <Accordion.Header.Item.Chevron />
              </Accordion.Header.Item.ActionGroup>
            </Accordion.Header.Item>
          </Accordion.Header.AccordionTrigger>
        </StyledAccordionHeader>
        <Accordion.Content>{children}</Accordion.Content>
      </Accordion.Container>
    </Accordion>
  );
}
