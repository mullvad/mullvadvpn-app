import { Accordion, type AccordionProps } from '../../../../../../../lib/components/accordion';
import {
  LocationListItemAccordionContent,
  LocationListItemAccordionTrigger,
  LocationListItemHeader,
} from './components';
import { LocationListItemAccordionProvider } from './LocationListItemAccordionContext';

export type LocationListItemAccordionProps = AccordionProps;

function LocationListItemAccordion({ children, ...props }: LocationListItemAccordionProps) {
  return (
    <LocationListItemAccordionProvider>
      <Accordion {...props}>{children}</Accordion>
    </LocationListItemAccordionProvider>
  );
}

const LocationListItemAccordionNamespace = Object.assign(LocationListItemAccordion, {
  Content: LocationListItemAccordionContent,
  Trigger: LocationListItemAccordionTrigger,
  Header: LocationListItemHeader,
  Container: Accordion.Container,
});

export { LocationListItemAccordionNamespace as LocationListItemAccordion };
