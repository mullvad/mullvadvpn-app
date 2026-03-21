import React from 'react';

import { Accordion } from '../../../../../../../../../lib/components/accordion';
import { useAccordionContext } from '../../../../../../../../../lib/components/accordion/AccordionContext';
import type { AccordionHeaderTriggerProps } from '../../../../../../../../../lib/components/accordion/components/accordion-header/components/accordion-header-trigger';
import { useLocationAccordionContext } from '../../LocationAccordionContext';

export type LocationAccordionTriggerProps = AccordionHeaderTriggerProps;

export function LocationAccordionTrigger({ children, ...props }: LocationAccordionTriggerProps) {
  const { setUserTriggeredExpand } = useLocationAccordionContext();
  const { expanded, onExpandedChange } = useAccordionContext();

  const handleClick = React.useCallback(
    (e: React.MouseEvent<HTMLButtonElement>) => {
      e.preventDefault();
      setUserTriggeredExpand(true);
      onExpandedChange?.(!expanded);
    },
    [setUserTriggeredExpand, onExpandedChange, expanded],
  );

  return (
    <Accordion.Header.Trigger onClick={handleClick} {...props}>
      {children}
    </Accordion.Header.Trigger>
  );
}
