import { Accordion } from '../../../../../../../../../../../lib/components/accordion';
import type { AccordionHeaderItemProps } from '../../../../../../../../../../../lib/components/accordion/components/accordion-header/components';
import { LocationAccordionHeaderItemTitle } from './components';

export type LocationAccordionHeaderItemProps = AccordionHeaderItemProps;

function LocationAccordionHeaderItem({ children, ...props }: LocationAccordionHeaderItemProps) {
  return <Accordion.Header.Item {...props}>{children}</Accordion.Header.Item>;
}

const LocationAccordionHeaderItemNamespace = Object.assign(LocationAccordionHeaderItem, {
  Title: LocationAccordionHeaderItemTitle,
  Chevron: Accordion.Header.Item.Chevron,
  ActionGroup: Accordion.Header.Item.ActionGroup,
});

export { LocationAccordionHeaderItemNamespace as LocationAccordionHeaderItem };
