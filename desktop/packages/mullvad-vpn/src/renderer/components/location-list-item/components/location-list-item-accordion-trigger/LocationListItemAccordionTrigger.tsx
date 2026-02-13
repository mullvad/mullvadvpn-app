import React from 'react';

import { Accordion } from '../../../../lib/components/accordion';
import { useAccordionContext } from '../../../../lib/components/accordion/AccordionContext';
import type { AccordionTriggerProps } from '../../../../lib/components/accordion/components';
import { useLocationListItemAccordionContext } from '../location-list-item-accordion/LocationListItemAccordionContext';

export type LocationListItemAccordionTriggerProps = AccordionTriggerProps;

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
    <Accordion.Trigger onClick={handleClick} {...props}>
      {children}
    </Accordion.Trigger>
  );
}
