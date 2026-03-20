import { Accordion } from '../../../../../../../../../../../lib/components/accordion';
import type { AccordionHeaderItemProps } from '../../../../../../../../../../../lib/components/accordion/components/accordion-header/components';
import { LocationListItemAccordionHeaderItemTitle } from './components';

export type LocationListItemAccordionHeaderItemProps = AccordionHeaderItemProps;

function LocationListItemAccordionHeaderItem({
  children,
  ...props
}: LocationListItemAccordionHeaderItemProps) {
  return <Accordion.Header.Item {...props}>{children}</Accordion.Header.Item>;
}

const LocationListItemAccordionHeaderItemNamespace = Object.assign(
  LocationListItemAccordionHeaderItem,
  {
    Title: LocationListItemAccordionHeaderItemTitle,
    Chevron: Accordion.Header.Item.Chevron,
    ActionGroup: Accordion.Header.Item.ActionGroup,
  },
);

export { LocationListItemAccordionHeaderItemNamespace as LocationListItemAccordionHeaderItem };
