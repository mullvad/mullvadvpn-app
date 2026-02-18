import React from 'react';

import { ScrollToAnchorId } from '../../../shared/ipc-types';
import { Accordion, AccordionProps } from '../../lib/components/accordion';
import { useHistory } from '../../lib/history';
import { SettingsAccordionHeader } from './components';
import { SettingsAccordionProvider } from './SettingsAccordionContext';

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
    <SettingsAccordionProvider anchorId={anchorId}>
      <Accordion
        expanded={expanded}
        onExpandedChange={handleOnExpandedChange}
        titleId={titleId}
        aria-labelledby={titleId}
        {...props}
      />
    </SettingsAccordionProvider>
  );
}

const SettingsAccordionNamespace = Object.assign(SettingsAccordion, {
  Container: Accordion.Container,
  Header: SettingsAccordionHeader,
  HeaderTrigger: Accordion.HeaderTrigger,
  HeaderItem: Accordion.HeaderItem,
  HeaderActionGroup: Accordion.HeaderActionGroup,
  HeaderTitle: Accordion.HeaderTitle,
  HeaderChevron: Accordion.HeaderChevron,
  Content: Accordion.Content,
});

export { SettingsAccordionNamespace as SettingsAccordion };
