import React from 'react';

import { ScrollToAnchorId } from '../../../shared/ipc-types';
import { useScrollToListItem } from '../../hooks';
import { Accordion, AccordionProps } from '../../lib/components/accordion';
import { useHistory } from '../../lib/history';

export type SettingsAccordion = Omit<AccordionProps, 'animation'> & {
  accordionId: string;
  anchorId?: ScrollToAnchorId;
};

function SettingsAccordion({ accordionId, anchorId, ...props }: SettingsAccordion) {
  const history = useHistory();
  const { location } = history;
  const { state } = location;
  const initialExpanded = location.state.expandedSections[accordionId];
  const [expanded, setExpanded] = React.useState(initialExpanded);
  const { ref, animation } = useScrollToListItem(anchorId);
  const titleId = React.useId();

  const handleOnExpandedChange = React.useCallback(
    (value: boolean) => {
      setExpanded(value);
      history.replace(location, {
        ...state,
        expandedSections: {
          ...state.expandedSections,
          [accordionId]: value,
        },
      });
    },
    [accordionId, history, location, state],
  );

  return (
    <Accordion
      ref={ref}
      tabIndex={-1}
      animation={animation}
      expanded={expanded}
      onExpandedChange={handleOnExpandedChange}
      titleId={titleId}
      aria-labelledby={titleId}
      {...props}
    />
  );
}

const SettingsAccordionNamespace = Object.assign(SettingsAccordion, {
  Trigger: Accordion.Trigger,
  HeaderItem: Accordion.HeaderItem,
  Content: Accordion.Content,
  Title: Accordion.Title,
  Icon: Accordion.Icon,
});

export { SettingsAccordionNamespace as SettingsAccordion };
