import React from 'react';

import { Accordion } from '../../../../../../../lib/components/accordion';
import { useAccordionContext } from '../../../../../../../lib/components/accordion/AccordionContext';
import type { AccordionHeaderTriggerProps } from '../../../../../../../lib/components/accordion/components/accordion-header/components/accordion-header-trigger';
import { useLocationListItemAccordionContext } from '../location-list-item-accordion/LocationListItemAccordionContext';

export type LocationListItemAccordionTriggerProps = AccordionHeaderTriggerProps;

export function LocationListItemAccordionTrigger({
  children,
  ...props
}: LocationListItemAccordionTriggerProps) {
  const { setUserTriggeredExpand } = useLocationListItemAccordionContext();
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
