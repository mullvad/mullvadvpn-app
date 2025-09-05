import React from 'react';

import { ListItem } from '../../list-item';
import { useAccordionContext } from '../AccordionContext';

export type AccordionTriggerProps = {
  children?: React.ReactNode;
} & React.ButtonHTMLAttributes<HTMLButtonElement>;

export function AccordionTrigger({ children }: AccordionTriggerProps) {
  const { contentId, triggerId, titleId, expanded, onExpandedChange } = useAccordionContext();

  const onClick = React.useCallback(
    (e: React.MouseEvent<HTMLButtonElement>) => {
      e.preventDefault();
      onExpandedChange?.(!expanded);
    },
    [onExpandedChange, expanded],
  );

  return (
    <ListItem.Trigger
      id={triggerId}
      aria-labelledby={titleId}
      aria-controls={contentId}
      aria-expanded={expanded ? 'true' : 'false'}
      onClick={onClick}>
      {children}
    </ListItem.Trigger>
  );
}
