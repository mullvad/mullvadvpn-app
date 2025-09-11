import React from 'react';

import { ScrollToAnchorId } from '../../../shared/ipc-types';
import { useScrollToListItem } from '../../hooks';
import { Accordion, AccordionProps } from '../../lib/components/accordion';
import { useHistory } from '../../lib/history';

export type SettingsAccordion = Omit<AccordionProps, 'animation'> & {
  anchorId?: ScrollToAnchorId;
};

function SettingsAccordion({ anchorId, ...props }: SettingsAccordion) {
  const { location } = useHistory();
  const initialExpanded = location.state.expandedSections['dns-blocker-setting'];
  const [expanded, setExpanded] = React.useState(initialExpanded);
  const { ref, animation } = useScrollToListItem(anchorId);

  return (
    <Accordion
      ref={ref}
      animation={animation}
      expanded={expanded}
      onExpandedChange={setExpanded}
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
