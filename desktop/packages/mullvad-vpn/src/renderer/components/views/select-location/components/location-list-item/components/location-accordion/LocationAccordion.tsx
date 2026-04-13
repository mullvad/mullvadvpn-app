import { Accordion, type AccordionProps } from '../../../../../../../lib/components/accordion';
import { LocationAccordionContent, LocationAccordionHeader } from './components';
import { LocationAccordionProvider } from './LocationAccordionContext';

export type LocationAccordionProps = AccordionProps;

function LocationAccordion({ children, ...props }: LocationAccordionProps) {
  return (
    <LocationAccordionProvider>
      <Accordion {...props}>{children}</Accordion>
    </LocationAccordionProvider>
  );
}

const LocationAccordionNamespace = Object.assign(LocationAccordion, {
  Content: LocationAccordionContent,
  Header: LocationAccordionHeader,
  Container: Accordion.Container,
});

export { LocationAccordionNamespace as LocationAccordion };
