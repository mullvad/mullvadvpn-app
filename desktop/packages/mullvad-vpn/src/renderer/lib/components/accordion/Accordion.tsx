import React from 'react';

import { ListItem } from '../list-item';
import { AccordionProvider } from './AccordionContext';
import {
  AccordionContainer,
  AccordionContent,
  AccordionHeader,
  AccordionHeaderChevron,
  AccordionHeaderItem,
  AccordionHeaderTitle,
  AccordionHeaderTrigger,
} from './components';

export type AccordionProps = React.PropsWithChildren<{
  expanded?: boolean;
  onExpandedChange?: (open: boolean) => void;
  disabled?: boolean;
  titleId?: string;
}>;

function Accordion({
  expanded = false,
  onExpandedChange,
  disabled,
  titleId,
  children,
}: AccordionProps) {
  return (
    <AccordionProvider
      titleId={titleId}
      expanded={expanded}
      onExpandedChange={onExpandedChange}
      disabled={disabled}>
      {children}
    </AccordionProvider>
  );
}

const AccordionNamespace = Object.assign(Accordion, {
  Container: AccordionContainer,
  HeaderTrigger: AccordionHeaderTrigger,
  Header: AccordionHeader,
  HeaderTitle: AccordionHeaderTitle,
  HeaderItem: AccordionHeaderItem,
  HeaderActionGroup: ListItem.ActionGroup,
  HeaderTrailingActions: ListItem.TrailingActions,
  HeaderTrailingAction: ListItem.TrailingAction,
  HeaderChevron: AccordionHeaderChevron,
  Content: AccordionContent,
});

export { AccordionNamespace as Accordion };
