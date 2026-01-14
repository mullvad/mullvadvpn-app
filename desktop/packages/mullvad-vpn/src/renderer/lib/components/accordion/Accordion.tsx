import React from 'react';

import { ListItem, ListItemProps } from '../list-item';
import { AccordionProvider } from './AccordionContext';
import {
  AccordionContent,
  AccordionHeaderItem,
  AccordionIcon,
  AccordionTitle,
  AccordionTrigger,
} from './components';

export type AccordionAnimation = 'flash' | 'dim';

export type AccordionProps = {
  expanded?: boolean;
  onExpandedChange?: (open: boolean) => void;
  disabled?: boolean;
  titleId?: string;
  animation?: AccordionAnimation;
  children?: React.ReactNode;
} & ListItemProps;

function Accordion({
  expanded = false,
  onExpandedChange: onOpenChange,
  disabled,
  animation,
  titleId: titleIdProp,
  children,
  ...props
}: AccordionProps) {
  const triggerId = React.useId();
  const contentId = React.useId();
  const titleId = React.useId();
  return (
    <AccordionProvider
      triggerId={triggerId}
      contentId={contentId}
      titleId={titleIdProp ?? titleId}
      expanded={expanded}
      onExpandedChange={onOpenChange}
      disabled={disabled}>
      <ListItem disabled={disabled} animation={animation} {...props}>
        {children}
      </ListItem>
    </AccordionProvider>
  );
}

const AccordionNamespace = Object.assign(Accordion, {
  Trigger: AccordionTrigger,
  HeaderItem: AccordionHeaderItem,
  Content: AccordionContent,
  Title: AccordionTitle,
  Icon: AccordionIcon,
});

export { AccordionNamespace as Accordion };
