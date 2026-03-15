import React from 'react';

import { ListItem } from '../../../../../list-item';
import { useAccordionContext } from '../../../../AccordionContext';

export type AccordionHeaderTriggerProps = {
  children?: React.ReactNode;
} & React.ComponentProps<'button'>;

export function AccordionHeaderTrigger({ children, ...props }: AccordionHeaderTriggerProps) {
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
      onClick={onClick}
      {...props}>
      {children}
    </ListItem.Trigger>
  );
}
