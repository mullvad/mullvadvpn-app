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
      animation={animation}
      expanded={expanded}
      onExpandedChange={handleOnExpandedChange}
      {...props}
    />
  );
}

const SettingsAccordionNamespace = Object.assign(SettingsAccordion, {
  Trigger: Accordion.Trigger,
  Header: Accordion.Header,
  Content: Accordion.Content,
  Title: Accordion.Title,
  Icon: Accordion.Icon,
});

export { SettingsAccordionNamespace as SettingsAccordion };
